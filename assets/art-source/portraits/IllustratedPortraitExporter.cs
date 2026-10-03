using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text.Json;

// Offline authoring: register selected masters, separate explicit material channels,
// and emit the established runtime source/mask/ink contract. No identity allocation.
public static class IllustratedPortraitExporter
{
    private const int Size = 512;
    private static string root = "";
    private static Dictionary<string, JsonElement> parts = new();
    private static readonly List<string> outputs = new();
    private static readonly Dictionary<string,string> frozen = new(StringComparer.Ordinal);

    public static void Run(string projectRoot, int catalogRevision)
    {
        if(catalogRevision!=1&&catalogRevision!=2) throw new InvalidDataException("Unsupported illustrated catalog revision.");
        bool release=catalogRevision==2;
        root = projectRoot;
        outputs.Clear();
        frozen.Clear();
        var baselinePath=Path.Combine(root,"assets/art-source/portraits/illustrated/export-manifest.json");
        if(File.Exists(baselinePath))
        {
            using var baseline=JsonDocument.Parse(File.ReadAllText(baselinePath));
            foreach(var entry in baseline.RootElement.GetProperty("exports").EnumerateArray())
                frozen.Add(entry.GetProperty("path").GetString()!,entry.GetProperty("sha256").GetString()!);
            VerifyFrozen();
        }
        using var manifest = JsonDocument.Parse(File.ReadAllText(Path.Combine(root,
            "assets/art-source/portraits/illustrated/source-manifest.json")));
        parts = manifest.RootElement.GetProperty("components").EnumerateArray()
            .ToDictionary(p => p.GetProperty("id").GetString()!, p => p.Clone());
        // Decode every selected input before replacing any previous proof exports.
        foreach (var id in parts.Keys)
        {
            if(!release&&parts[id].TryGetProperty("introduced_catalog_revision",out var introduced)&&introduced.GetInt32()>1) continue;
            using var prepared = Prepare(id); CheckAlpha(prepared,id);
        }
        foreach (var face in release?new[] { "face_oval", "face_tapered", "face_square", "face_round" }:new[]{"face_oval","face_tapered"})
        {
            using var head = Prepare(face + "_master");
            using var mouth = Prepare("mouth_neutral");
            Over(head,mouth);
            Shaded(head,$"faces/{face}","");
            foreach(var nose in release?new[]{"nose_straight","nose_upturned","nose_aquiline"}:new[]{"nose_straight","nose_upturned"})
            {
                using var source=Prepare(nose);
                ExportNose(source,$"noses/{face}/{nose}");
            }
            foreach(var eyes in release?new[]{"eyes_open","eyes_lidded","eyes_round"}:new[]{"eyes_open","eyes_lidded"})
            {
                using var source=Prepare(eyes);
                ExportEyes(source,$"eyes/{face}/{eyes}");
            }
            foreach(var hair in release?new[]{"hair_cropped","hair_coiled","hair_wavy","hair_braided","hair_topknot","hair_long","hair_undercut"}:new[]{"hair_cropped","hair_coiled"})
            foreach(var side in new[]{"rear","front"})
            {
                using var source=Prepare(hair+"_"+side);
                Shaded(source,$"hair/{face}/{hair}",side+"_");
            }
        }
        using(var combined=Prepare("support"))
        {
            var separated=SplitSupport(combined);
            using(var neck=separated.Item1) Shaded(neck,"supporting/neck","");
            using(var clothing=separated.Item2) Shaded(clothing,"supporting/shoulders","");
        }
        // Identity-neutral fallback drawings remain deliberate simple silhouettes.
        foreach(var file in new[]{"adult_fallback.png","child_silhouette.png","unknown_silhouette.png"})
        {
            var relative="assets/portraits/supporting/"+file;
            if(!File.Exists(Path.Combine(root,relative))) throw new InvalidDataException("Missing fallback: "+relative);
            outputs.Add(relative);
        }
        var records=outputs.OrderBy(p=>p,StringComparer.Ordinal).Select(p=>new {
            path=p, sha256=Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(Path.Combine(root,p))))
        }).ToArray();
        VerifyFrozen();
        if(release) File.WriteAllText(Path.Combine(root,"assets/art-source/portraits/illustrated/export-manifest-v2.json"),
            JsonSerializer.Serialize(new { schema_version=1, catalog_revision=2, rig_id="human_bust_front_v1", frozen_v1_count=frozen.Count, exports=records },new JsonSerializerOptions{WriteIndented=true})+Environment.NewLine);
        Console.WriteLine($"Illustrated export complete: {outputs.Count} aligned runtime PNGs; {frozen.Count} frozen G02 bytes preserved.");
    }

    private static Bitmap Prepare(string id)
    {
        if(!parts.TryGetValue(id,out var part)) throw new InvalidDataException("Missing source component: "+id);
        var s=part.GetProperty("source_rect").EnumerateArray().Select(v=>v.GetInt32()).ToArray();
        var d=part.GetProperty("destination_rect").EnumerateArray().Select(v=>v.GetSingle()).ToArray();
        using var input=new Bitmap(Path.Combine(root,"assets/art-source/portraits/illustrated",part.GetProperty("file").GetString()!));
        if(s[0]<0||s[1]<0||s[0]+s[2]>input.Width||s[1]+s[3]>input.Height)
            throw new InvalidDataException("Source registration outside image: "+id);
        var output=new Bitmap(Size,Size,PixelFormat.Format32bppArgb);
        using var g=Graphics.FromImage(output);
        g.CompositingMode=CompositingMode.SourceCopy;
        g.InterpolationMode=InterpolationMode.HighQualityBicubic;
        g.PixelOffsetMode=PixelOffsetMode.HighQuality;
        g.DrawImage(input,new RectangleF(d[0],d[1],d[2],d[3]),new Rectangle(s[0],s[1],s[2],s[3]),GraphicsUnit.Pixel);
        return output;
    }

    private static void Shaded(Bitmap master,string folder,string prefix)
    {
        using var baseSource=Blank(); using var shadowSource=Blank(); using var highlightSource=Blank(); using var ink=Blank();
        for(int y=0;y<Size;y++) for(int x=0;x<Size;x++)
        {
            var p=master.GetPixel(x,y);
            double a=Coverage(p.A); if(a==0) continue;
            double v=Gray(p)/255.0;
            double wb=0,ws=0,wh=0,wi=0;
            if(v<.17){wi=1-v/.17;ws=1-wi;}
            else if(v<.76){wb=(v-.17)/.59;ws=1-wb;}
            else {wh=Math.Clamp((v-.76)/.24,0,1);wb=1-wh;}
            // Solve alpha from the last plane backwards, so straight-alpha over
            // reproduces the reviewed ramp mixture even along translucent edges.
            double ai=a*wi, remaining=1-ai;
            double ah=remaining>0?a*wh/remaining:0; remaining*=1-ah;
            double ash=remaining>0?a*ws/remaining:0; remaining*=1-ash;
            double ab=remaining>0?a*wb/remaining:0;
            SetWhite(baseSource,x,y,ab); SetWhite(shadowSource,x,y,ash); SetWhite(highlightSource,x,y,ah);
            if(ai>0) ink.SetPixel(x,y,Color.FromArgb(Byte(ai),24,22,25));
        }
        SavePair(baseSource,folder+"/"+prefix+"base");
        SavePair(shadowSource,folder+"/"+prefix+"shadow");
        SavePair(highlightSource,folder+"/"+prefix+"highlight");
        Save(ink,folder+"/"+prefix+"ink.png");
    }

    private static void ExportNose(Bitmap master,string stem)
    {
        using var source=Blank();
        for(int y=0;y<Size;y++) for(int x=0;x<Size;x++)
        {
            var p=master.GetPixel(x,y); if(p.A==0) continue;
            int value=Gray(p);
            source.SetPixel(x,y,Color.FromArgb(Byte(Coverage(p.A)*.70),value,value,value));
        }
        SavePair(source,stem);
    }

    private static void ExportEyes(Bitmap master,string stem)
    {
        using var whites=Blank(); using var iris=Blank(); using var ink=Blank();
        for(int y=0;y<Size;y++) for(int x=0;x<Size;x++)
        {
            var p=master.GetPixel(x,y); double a=Coverage(p.A); if(a==0) continue;
            double neutral=Math.Max(p.R,p.B), pigment=Math.Max(0,p.G-neutral);
            double light=neutral+pigment;
            double weight=light>0?pigment/light:0;
            int value=Math.Clamp((int)Math.Round(light),0,255);
            if(weight==0)
            {
                var target=value>190?whites:ink;
                target.SetPixel(x,y,Color.FromArgb(Byte(a),value,value,value));
                continue;
            }
            double inkAlpha=a*(1-weight);
            double irisAlpha=inkAlpha<1?a*weight/(1-inkAlpha):0;
            iris.SetPixel(x,y,Color.FromArgb(Byte(irisAlpha),value,value,value));
            if(inkAlpha>0) ink.SetPixel(x,y,Color.FromArgb(Byte(inkAlpha),value,value,value));
        }
        Save(whites,stem+"_whites.png"); SavePair(iris,stem+"_iris"); Save(ink,stem+"_ink.png");
    }

    private static Tuple<Bitmap,Bitmap> SplitSupport(Bitmap master)
    {
        var neck=Blank(); var cloth=Blank();
        for(int y=0;y<Size;y++) for(int x=0;x<Size;x++)
        {
            var p=master.GetPixel(x,y); double a=Coverage(p.A); if(a==0) continue;
            double neutral=Math.Max(p.G,p.B), pigment=Math.Max(0,p.R-neutral);
            double weight=p.R>0?pigment/p.R:0;
            double na=a*weight, ca=na<1?a*(1-weight)/(1-na):0;
            if(na>0) neck.SetPixel(x,y,Color.FromArgb(Byte(na),p.R,p.R,p.R));
            int gray=(int)neutral;
            if(ca>0) cloth.SetPixel(x,y,Color.FromArgb(Byte(ca),gray,gray,gray));
        }
        return Tuple.Create(neck,cloth);
    }

    private static void SavePair(Bitmap source,string stem)
    {
        using var mask=Blank();
        for(int y=0;y<Size;y++) for(int x=0;x<Size;x++)
            if(source.GetPixel(x,y).A>0) mask.SetPixel(x,y,Color.White);
        Save(source,stem+"_source.png"); Save(mask,stem+"_mask.png");
    }

    private static void Save(Bitmap image,string relative)
    {
        var asset="assets/portraits/"+relative;
        if(frozen.ContainsKey(asset)) { outputs.Add(asset); return; }
        var path=Path.Combine(root,asset); Directory.CreateDirectory(Path.GetDirectoryName(path)!);
        image.Save(path,ImageFormat.Png); outputs.Add(asset);
    }

    private static void VerifyFrozen()
    {
        foreach(var entry in frozen)
        {
            var path=Path.Combine(root,entry.Key);
            if(!File.Exists(path)||Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(path)))!=entry.Value)
                throw new InvalidDataException("Frozen G02 portrait asset changed or disappeared: "+entry.Key);
        }
    }

    private static void CheckAlpha(Bitmap image,string id)
    {
        int count=0;
        for(int y=0;y<Size;y++) for(int x=0;x<Size;x++)
        {
            var p=image.GetPixel(x,y); if(p.A==0) continue;
            if(x<40||x>=472||y<24||y>=496) throw new InvalidDataException("Registered source escapes safe rectangle: "+id);
            count++;
        }
        if(count==0) throw new InvalidDataException("Empty registered component: "+id);
    }

    private static Bitmap Blank()=>new Bitmap(Size,Size,PixelFormat.Format32bppArgb);
    private static int Gray(Color p)=>Math.Clamp((int)Math.Round(p.R*.2126+p.G*.7152+p.B*.0722),0,255);
    private static double Coverage(byte alpha)=>alpha>=240?1:alpha/255.0;
    private static int Byte(double value)=>Math.Clamp((int)Math.Round(value*255,MidpointRounding.AwayFromZero),0,255);
    private static void SetWhite(Bitmap image,int x,int y,double alpha)
    {
        int a=Byte(alpha); if(a>0) image.SetPixel(x,y,Color.FromArgb(a,255,255,255));
    }
    private static void Over(Bitmap destination,Bitmap source)
    {
        using var g=Graphics.FromImage(destination); g.CompositingMode=CompositingMode.SourceOver; g.DrawImageUnscaled(source,0,0);
    }
}
