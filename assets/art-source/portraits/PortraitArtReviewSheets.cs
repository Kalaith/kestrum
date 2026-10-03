using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Linq;
using System.Text.Json;

// Release review uses the same exported planes, straight-alpha compositor and
// premultiplied area reduction as the G02 acceptance harness.
public static partial class PortraitArtSource
{
    public static void WriteParityFixtures(string projectRoot,string catalogJson)
    {
        root=projectRoot;PixelCache.Clear();
        using var document=JsonDocument.Parse(catalogJson);var catalog=document.RootElement;
        string face="face_oval",nose="nose_upturned",eyes="eyes_lidded",hair="hair_coiled";
        string skin="skin_ochre",hairPalette="hair_flax",iris="eyes_hazel";
        using var portrait=Render(catalog,face,nose,eyes,hair,skin,hairPalette,iris);
        var files=new List<object>();
        foreach(int size in new[]{128,256})
        {
            string relative=$"assets/art-source/portraits/illustrated/parity_v1_{size}.png";
            using var reduced=DownsampleArea(portrait,size,size);reduced.Save(Out(relative),ImageFormat.Png);
            files.Add(new{path=relative,width=size,height=size,sha256=HashFile(Out(relative))});
        }
        string Key(string group,string id)=>catalog.GetProperty(group).EnumerateArray().Single(p=>Id(p,"id")==id).GetProperty("visual_key").GetString()!;
        var keys=new[]{catalog.GetProperty("rig").GetProperty("visual_key").GetString()!,Key("faces",face),Key("noses",nose),Key("eyes",eyes),Key("hair",hair),Key("skin_palettes",skin),Key("hair_palettes",hairPalette),Key("eye_palettes",iris)};
        var signature=string.Join("|",keys.Select(k=>$"{k.Length}:{k}"));
        var provenance=new{
            schema_version=1, exporter="PortraitArtSource reference straight-alpha compositor",exporter_version="illustrated-ramp-v1",
            command="scripts/export_illustrated_portraits.ps1 -ParityOnly",source_revision="frozen G02 runtime exports",
            frozen_exports_manifest="export-manifest.json",frozen_exports_manifest_sha256=HashFile(Out("assets/art-source/portraits/illustrated/export-manifest.json")),
            descriptor=new{schema_version=1,catalog_revision=1,allocation_revision=1,rig_id="human_bust_front_v1",face_id=face,nose_id=nose,eyes_id=eyes,hair_id=hair,skin_palette_id=skin,hair_palette_id=hairPalette,eye_palette_id=iris,signature},
            composition="Per-plane grayscale-red multiplied RGB tint; source alpha * mask red * mask alpha * tint alpha; straight-alpha Porter-Duff over. Order: rear hair, shoulders, neck, face fills, eyes whites/iris/ink, nose, face ink/mouth, front hair. Shaded assemblies use base, shadow, highlight, ink (face ink deferred). Shoulder shadow/base/highlight RGB: [39,51,58]/[84,101,108]/[153,164,157].",
            reduction="Exact area filtering in premultiplied-alpha space from 512x512, then unpremultiplication. Exactly zero accumulated alpha has zero RGB; positive coverage rounded to byte-alpha zero can retain unused RGB. Clamped byte conversion rounds halfway away from zero after each over operation and reduction. Compare alpha and premultiplied RGB, not unused transparent RGB.",
            files
        };
        File.WriteAllText(Out("assets/art-source/portraits/illustrated/parity_v1.json"),JsonSerializer.Serialize(provenance,new JsonSerializerOptions{WriteIndented=true})+Environment.NewLine);
        Console.WriteLine("Stable v1 parity fixtures written at 128px and 256px; descriptor/provenance: assets/art-source/portraits/illustrated/parity_v1.json");
    }

    private static string HashFile(string path)=>Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(File.ReadAllBytes(path)));

    private static void ValidateReleaseMatrix(JsonElement catalog)
    {
        var skins=Ramps(catalog,"skin_palettes");
        var hairs=Ramps(catalog,"hair_palettes");
        var eyes=Ramps(catalog,"eye_palettes");
        int tuples=0,legal=0,composites=0;
        foreach(var tuple in catalog.GetProperty("geometry").EnumerateArray())
        {
            var face=Id(tuple,"face_id"); var nose=Id(tuple,"nose_id");
            var eye=Id(tuple,"eyes_id"); var hair=Id(tuple,"hair_id");
            var choices=hairs.Where(p=>hair=="hair_bald"?p.Id=="none":p.Id!="none").ToArray();
            var variants=new[]{
                (skins[tuples%skins.Length],choices[tuples%choices.Length],eyes[tuples%eyes.Length]),
                (skins.MinBy(Luminance)!,choices.MinBy(Luminance)!,eyes.MinBy(Luminance)!),
                (skins.MaxBy(Luminance)!,choices.MaxBy(Luminance)!,eyes.MaxBy(Luminance)!)
            };
            foreach(var variant in variants)
            {
                var pixels=CompositePixels(face,nose,eye,hair,variant.Item1,variant.Item2,variant.Item3);
                AssertCanvas(pixels,$"{face}/{nose}/{eye}/{hair}",false);
                composites++;
            }
            ValidateEyeVisibility(face,eye,hair,choices[0]);
            legal+=skins.Length*choices.Length*eyes.Length;
            tuples++;
        }
        if(tuples!=288||legal!=37152||skins.Length!=6||hairs.Length!=7||eyes.Length!=4)
            throw new InvalidDataException($"Release matrix mismatch: {tuples} geometry, {legal} legal colors.");
        Console.WriteLine($"Release pixel matrix passed: {tuples} geometry tuples; {composites} baseline/extreme composites; {legal} legal descriptors counted (not all rasterized). Both irises retain at least 45% of their authored coverage.");
    }

    private static void ValidateEyeVisibility(string face,string eye,string hair,Ramp ramp)
    {
        if(hair=="hair_bald") return;
        var front=new byte[Size*Size*4];
        DrawShaded(front,Out($"assets/portraits/hair/{face}/{hair}"),"front_",ramp);
        var source=ReadPngPixels(Out($"assets/portraits/eyes/{face}/{eye}_iris_source.png"));
        var mask=ReadPngPixels(Out($"assets/portraits/eyes/{face}/{eye}_iris_mask.png"));
        var total=new double[2]; var visible=new double[2];
        for(int y=0;y<Size;y++) for(int x=0;x<Size;x++)
        {
            int at=(y*Size+x)*4, side=x<256?0:1;
            double a=source[at+3]/255.0*mask[at+2]/255.0*mask[at+3]/255.0;
            total[side]+=a; visible[side]+=a*(1-front[at+3]/255.0);
        }
        for(int side=0;side<2;side++)
            if(total[side]<1||visible[side]/total[side]<.45)
                throw new InvalidDataException($"Hair obscures iris {side}: {face}/{eye}/{hair}, retained {visible[side]/Math.Max(1,total[side]):P1}.");
    }

    private static Ramp[] Ramps(JsonElement catalog,string key)=>catalog.GetProperty(key).EnumerateArray()
        .Select(p=>ReadRamp(catalog.GetProperty(key),Id(p,"id"))).ToArray();
    private static string Id(JsonElement value,string key)=>value.GetProperty(key).GetString()!;
    private static double Luminance(Ramp p)=>p.Base[0]*.2126+p.Base[1]*.7152+p.Base[2]*.0722;
    private static string Short(string id)=>id[(id.IndexOf('_')+1)..];

    private static Bitmap Render(JsonElement catalog,string face,string nose,string eyes,string hair,string skin,string hairPalette,string iris)
        =>Composite(face,nose,eyes,hair,ReadRamp(catalog.GetProperty("skin_palettes"),skin),
            ReadRamp(catalog.GetProperty("hair_palettes"),hairPalette),ReadRamp(catalog.GetProperty("eye_palettes"),iris));

    private static void GenerateReleaseContactSheets(JsonElement catalog)
    {
        WriteRepresentativeSheet(catalog);
        WriteGeometrySheet(catalog);
        WritePaletteSheet(catalog);
    }

    private static void WriteRepresentativeSheet(JsonElement catalog)
    {
        var faces=catalog.GetProperty("faces").EnumerateArray().Select(p=>Id(p,"id")).ToArray();
        var noses=catalog.GetProperty("noses").EnumerateArray().Select(p=>Id(p,"id")).ToArray();
        var eyes=catalog.GetProperty("eyes").EnumerateArray().Select(p=>Id(p,"id")).ToArray();
        var hairs=catalog.GetProperty("hair").EnumerateArray().Select(p=>Id(p,"id")).ToArray();
        var skins=Ramps(catalog,"skin_palettes"); var colors=Ramps(catalog,"hair_palettes").Where(p=>p.Id!="none").ToArray();
        var irises=Ramps(catalog,"eye_palettes");
        using var sheet=new Bitmap(1120,1160,PixelFormat.Format32bppArgb);
        using var g=Graphics.FromImage(sheet); g.Clear(Color.FromArgb(244,239,229));
        using var title=new Font("Georgia",25,FontStyle.Bold,GraphicsUnit.Pixel);
        using var label=new Font("Segoe UI",12,FontStyle.Regular,GraphicsUnit.Pixel);
        using var ink=new SolidBrush(Color.FromArgb(46,40,38));
        g.DrawString("KESTRUM - ILLUSTRATED PORTRAIT RELEASE MATRIX",title,ink,18,12);
        g.DrawString("Actual exported layers. Each example shows 40, 64 and 128 px from left to right. No human recognition claim.",label,ink,20,49);
        for(int i=0;i<24;i++)
        {
            int group=i/8; var hair=hairs[i%hairs.Length]; var face=faces[(i+group)%faces.Length];
            var eye=eyes[group%eyes.Length]; var nose=noses[(i+group)%noses.Length];
            var skin=skins[(i+group)%skins.Length].Id;
            var color=hair=="hair_bald"?"none":colors[(i+2*group)%colors.Length].Id;
            var iris=irises[i%irises.Length].Id;
            int x=(i%4)*280,y=80+(i/4)*180;
            using var background=new SolidBrush(i%2==0?Color.FromArgb(42,47,54):Color.FromArgb(237,225,205));
            using var text=new SolidBrush(i%2==0?Color.FromArgb(235,234,224):Color.FromArgb(46,40,38));
            g.FillRectangle(background,x+4,y+2,272,172);
            g.DrawString($"{Short(face)} / {Short(eye)} / {Short(hair)}",label,text,x+10,y+6);
            g.DrawString($"{Short(skin)} / {Short(color)} / {Short(iris)}",label,text,x+10,y+22);
            using var portrait=Render(catalog,face,nose,eye,hair,skin,color,iris);
            int[] sizes={40,64,128}; int[] offsets={12,60,132};
            for(int j=0;j<3;j++)
            {
                using var reduced=DownsampleArea(portrait,sizes[j],sizes[j]);
                g.DrawImageUnscaled(reduced,x+offsets[j],y+166-sizes[j]);
            }
        }
        sheet.Save(Out("docs/verification/portrait_contact_sheet.png"),ImageFormat.Png);
    }

    private static void WriteGeometrySheet(JsonElement catalog)
    {
        using var sheet=new Bitmap(1920,1162,PixelFormat.Format32bppArgb);
        using var g=Graphics.FromImage(sheet); g.Clear(Color.FromArgb(244,239,229));
        using var title=new Font("Georgia",23,FontStyle.Bold,GraphicsUnit.Pixel);
        using var label=new Font("Segoe UI",11,FontStyle.Regular,GraphicsUnit.Pixel);
        using var ink=new SolidBrush(Color.FromArgb(46,40,38));
        g.DrawString("KESTRUM - ALL 288 GEOMETRY FITS AT 64 PX",title,ink,18,10);
        g.DrawString("Index order and exact tuples: assets/art-source/portraits/illustrated/geometry-review.json. Fixed fair/ebony/hazel palette isolates shape and fit.",label,ink,20,42);
        var records=new List<object>(); int index=0;
        foreach(var tuple in catalog.GetProperty("geometry").EnumerateArray())
        {
            var face=Id(tuple,"face_id");var nose=Id(tuple,"nose_id");var eyes=Id(tuple,"eyes_id");var hair=Id(tuple,"hair_id");
            int x=(index%24)*80,y=70+(index/24)*91;
            using var back=new SolidBrush(index%2==0?Color.FromArgb(42,47,54):Color.FromArgb(237,225,205));
            g.FillRectangle(back,x+2,y,76,72);
            using var portrait=Render(catalog,face,nose,eyes,hair,"skin_fair",hair=="hair_bald"?"none":"hair_ebony","eyes_hazel");
            using var reduced=DownsampleArea(portrait,64,64);g.DrawImageUnscaled(reduced,x+8,y+4);
            g.DrawString(index.ToString("D3"),label,ink,x+28,y+74);
            records.Add(new{index,face_id=face,nose_id=nose,eyes_id=eyes,hair_id=hair});index++;
        }
        sheet.Save(Out("docs/verification/portrait_geometry_matrix.png"),ImageFormat.Png);
        File.WriteAllText(Out("assets/art-source/portraits/illustrated/geometry-review.json"),JsonSerializer.Serialize(records,new JsonSerializerOptions{WriteIndented=true})+Environment.NewLine);
    }

    private static void WritePaletteSheet(JsonElement catalog)
    {
        var skins=Ramps(catalog,"skin_palettes");var hairs=Ramps(catalog,"hair_palettes").Where(p=>p.Id!="none").ToArray();
        var eyes=Ramps(catalog,"eye_palettes");
        using var sheet=new Bitmap(984,902,PixelFormat.Format32bppArgb);
        using var g=Graphics.FromImage(sheet);g.Clear(Color.FromArgb(244,239,229));
        using var title=new Font("Georgia",23,FontStyle.Bold,GraphicsUnit.Pixel);
        using var label=new Font("Segoe UI",12,FontStyle.Regular,GraphicsUnit.Pixel);
        using var ink=new SolidBrush(Color.FromArgb(46,40,38));
        g.DrawString("KESTRUM - PALETTE READABILITY ON LIGHT AND DARK",title,ink,16,10);
        g.DrawString("Each cell: identical 64px portrait on both surfaces. Round face / round eyes / coiled hair; gray iris.",label,ink,18,43);
        for(int col=0;col<hairs.Length;col++)g.DrawString(Short(hairs[col].Id),label,ink,120+col*142,68);
        for(int row=0;row<skins.Length;row++)
        {
            g.DrawString(Short(skins[row].Id),label,ink,8,113+row*101);
            for(int col=0;col<hairs.Length;col++)
            {
                using var portrait=Render(catalog,"face_round","nose_upturned","eyes_round","hair_coiled",skins[row].Id,hairs[col].Id,"eyes_gray");
                using var reduced=DownsampleArea(portrait,64,64);
                for(int side=0;side<2;side++)
                {
                    int x=116+col*142+side*68,y=89+row*101;
                    using var back=new SolidBrush(side==0?Color.FromArgb(237,225,205):Color.FromArgb(42,47,54));
                    g.FillRectangle(back,x,y,66,84);g.DrawImageUnscaled(reduced,x+1,y+10);
                }
            }
        }
        g.DrawString("Iris colors at 128px (umber skin / silver hair)",label,ink,18,710);
        for(int i=0;i<eyes.Length;i++)
        {
            int x=145+i*190;
            using var portrait=Render(catalog,"face_round","nose_upturned","eyes_round","hair_long","skin_umber","hair_silver",eyes[i].Id);
            using var reduced=DownsampleArea(portrait,128,128);g.DrawImageUnscaled(reduced,x,742);
            g.DrawString(Short(eyes[i].Id),label,ink,x+39,878);
        }
        sheet.Save(Out("docs/verification/portrait_palette_edges.png"),ImageFormat.Png);
    }
}
