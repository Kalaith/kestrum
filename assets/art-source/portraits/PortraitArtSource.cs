using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text.Json;

// Original Kestrum portrait vector artwork and deterministic RGBA compositor.
// All coordinates are authored on the catalog's untrimmed 512x512 rig.
public static partial class PortraitArtSource
{
    private const int Size = 512;
    private const int Supersample = 2;
    private static readonly Color Ink = Color.FromArgb(255, 54, 39, 38);
    private static readonly Color SoftInk = Color.FromArgb(228, 90, 59, 49);
    private static readonly Color WarmInk = Color.FromArgb(238, 111, 67, 50);
    private static readonly Color White = Color.FromArgb(255, 255, 255, 255);
    private static string root = "";
    private static readonly Dictionary<string, byte[]> PixelCache = new(StringComparer.OrdinalIgnoreCase);

    private sealed record Ramp(string Id, byte[] Shadow, byte[] Base, byte[] Highlight);
    private sealed record Materials(string Directory, string Prefix = "");
    private sealed record Sample(string Label, string Face, string Nose, string Eyes, string Hair,
        string Skin, string HairPalette, string Iris, bool DarkBackground);

    public static void Run(string projectRoot, string catalogJson, bool validateOnly = false)
    {
        root = projectRoot;
        var catalog = JsonDocument.Parse(catalogJson);
        ValidateContract(catalog.RootElement);
        if (!validateOnly) GenerateAllAssets();
        ValidateEveryTupleAndPalette(catalog.RootElement);
        ValidateExportSet();
        if (!validateOnly) GenerateContactSheet(catalog.RootElement);
        Console.WriteLine("Portrait art validation completed; actual matrix and export coverage are reported above.");
        if (!validateOnly) Console.WriteLine("Contact sheet: docs/verification/portrait_contact_sheet.png");
    }

    public static void ReviewExistingExports(string projectRoot, string catalogJson)
    {
        root = projectRoot;
        PixelCache.Clear();
        using var catalog = JsonDocument.Parse(catalogJson);
        ValidateContract(catalog.RootElement);
        ValidateEveryTupleAndPalette(catalog.RootElement);
        ValidateExportSet();
        GenerateContactSheet(catalog.RootElement);
    }

    private static void ValidateContract(JsonElement catalog)
    {
        var rig = catalog.GetProperty("rig");
        if (rig.GetProperty("width").GetInt32() != Size || rig.GetProperty("height").GetInt32() != Size)
            throw new InvalidDataException("The authored source is frozen to the catalog's 512x512 rig.");
        var safe = rig.GetProperty("safe_rect").EnumerateArray().Select(value => value.GetInt32()).ToArray();
        if (!safe.SequenceEqual(new[] { 40, 24, 472, 496 }))
            throw new InvalidDataException("The catalog safe rectangle changed; review the frozen art source before regeneration.");
        var anchors = rig.GetProperty("anchors");
        CheckAnchor(anchors, "crown", 256, 32);
        CheckAnchor(anchors, "scalp_center", 256, 116);
        CheckAnchor(anchors, "eye_line", 256, 236);
        CheckAnchor(anchors, "nose_bridge", 256, 270);
        CheckAnchor(anchors, "nose_tip", 256, 316);
        CheckAnchor(anchors, "mouth_center", 256, 354);
        CheckAnchor(anchors, "chin", 256, 408);
        int revision=catalog.GetProperty("catalog_revision").GetInt32();
        if(revision!=1&&revision!=2) throw new InvalidDataException("Unsupported authored catalog revision.");
        bool release=revision==2;
        CheckIds(catalog,"faces",release?new[]{"face_oval","face_tapered","face_square","face_round"}:new[]{"face_oval","face_tapered"});
        CheckIds(catalog,"noses",release?new[]{"nose_straight","nose_upturned","nose_aquiline"}:new[]{"nose_straight","nose_upturned"});
        CheckIds(catalog,"eyes",release?new[]{"eyes_open","eyes_lidded","eyes_round"}:new[]{"eyes_open","eyes_lidded"});
        CheckIds(catalog,"hair",release?new[]{"hair_bald","hair_cropped","hair_coiled","hair_wavy","hair_braided","hair_topknot","hair_long","hair_undercut"}:new[]{"hair_bald","hair_cropped","hair_coiled"});
        CheckIds(catalog,"skin_palettes",release?new[]{"skin_fair","skin_ochre","skin_umber","skin_porcelain","skin_sand","skin_deep"}:new[]{"skin_fair","skin_ochre","skin_umber"});
        CheckIds(catalog,"hair_palettes",release?new[]{"none","hair_ebony","hair_copper","hair_flax","hair_silver","hair_indigo","hair_chestnut"}:new[]{"none","hair_ebony","hair_copper","hair_flax"});
        CheckIds(catalog,"eye_palettes",release?new[]{"eyes_brown","eyes_hazel","eyes_blue","eyes_gray"}:new[]{"eyes_brown","eyes_hazel"});
    }

    private static void CheckAnchor(JsonElement anchors, string name, int x, int y)
    {
        var actual = anchors.GetProperty(name).EnumerateArray().Select(value => value.GetInt32()).ToArray();
        if (!actual.SequenceEqual(new[] { x, y }))
            throw new InvalidDataException($"Anchor {name} changed; update and review the frozen art source.");
    }

    private static void CheckIds(JsonElement catalog, string group, params string[] expected)
    {
        var actual = catalog.GetProperty(group).EnumerateArray()
            .Select(item => item.GetProperty("id").GetString() ?? "").ToArray();
        if (!actual.OrderBy(v=>v,StringComparer.Ordinal).SequenceEqual(expected.OrderBy(v=>v,StringComparer.Ordinal)))
            throw new InvalidDataException($"Catalog {group} IDs differ from the authored matrix.");
    }

    private static string Out(string relative) => Path.Combine(root, relative.Replace('/', Path.DirectorySeparatorChar));

    private static void GenerateAllAssets()
    {
        foreach (var face in new[] { "face_oval", "face_tapered" })
        {
            WriteFace(face);
            foreach (var nose in new[] { "nose_straight", "nose_upturned" }) WriteNose(face, nose);
            foreach (var eyes in new[] { "eyes_open", "eyes_lidded" }) WriteEyes(face, eyes);
            foreach (var hair in new[] { "hair_cropped", "hair_coiled" }) WriteHair(face, hair);
        }
        WriteSupporting();
    }

    private static void WriteFace(string face)
    {
        var folder = Out($"assets/portraits/faces/{face}");
        var silhouette = FaceSilhouette(face);
        SavePair(folder, "base", new[] { silhouette }, new[] { silhouette }, "face-base");
        SavePair(folder, "shadow", FaceShadows(face), FaceShadows(face), "face-shadow");
        SavePair(folder, "highlight", FaceHighlights(face), FaceHighlights(face), "face-highlight");
        SaveInk(folder, "ink.png", g => DrawFaceInk(g, face));
    }

    private static void WriteNose(string face, string nose)
    {
        var folder = Out($"assets/portraits/noses/{face}");
        var data = NoseShading(face, nose);
        SavePair(folder, nose + "_", data, data, nose);
    }

    private static void WriteEyes(string face, string eyes)
    {
        var folder = Out($"assets/portraits/eyes/{face}");
        var whites = EyeWhites(face, eyes);
        SaveInk(folder, eyes + "_whites.png", g => FillMany(g, whites, Color.FromArgb(255, 244, 232, 211)));
        var iris = IrisShapes(face, eyes);
        SavePair(folder, eyes + "_iris", iris, iris, eyes + "-iris");
        SaveInk(folder, eyes + "_ink.png", g => DrawEyeInk(g, face, eyes));
    }

    private static void WriteHair(string face, string hair)
    {
        var folder = Out($"assets/portraits/hair/{face}/{hair}");
        foreach (var side in new[] { "rear", "front" })
        {
            var baseShapes = HairShapes(face, hair, side, "base");
            SavePair(folder, side + "_base", baseShapes, baseShapes, hair + "-" + side);
            var shadowShapes = HairShapes(face, hair, side, "shadow");
            SavePair(folder, side + "_shadow", shadowShapes, shadowShapes, hair + "-shadow");
            var highlightShapes = HairShapes(face, hair, side, "highlight");
            SavePair(folder, side + "_highlight", highlightShapes, highlightShapes, hair + "-highlight");
            SaveInk(folder, side + "_ink.png", g => DrawHairInk(g, face, hair, side));
        }
    }

    private static void SavePair(string folder, string prefix, string[] sourcePaths, string[] maskPaths, string kind)
    {
        Directory.CreateDirectory(folder);
        var stem = prefix.EndsWith("_", StringComparison.Ordinal) ? prefix : prefix + "_";
        SaveRaster(Path.Combine(folder, stem + "source.png"), g => FillShaded(g, sourcePaths, kind));
        var maskAlpha = kind == "face-shadow" ? (byte)88 : kind == "face-highlight" ? (byte)68 : (byte)255;
        SaveRaster(Path.Combine(folder, stem + "mask.png"), g => FillMany(g, maskPaths, Color.FromArgb(maskAlpha, 255, 255, 255)));
    }

    private static void SaveInk(string folder, string file, Action<Graphics> draw)
    {
        Directory.CreateDirectory(folder);
        SaveRaster(Path.Combine(folder, file), draw);
    }

    private static void SaveRaster(string path, Action<Graphics> draw)
    {
        using var large = new Bitmap(Size * Supersample, Size * Supersample, PixelFormat.Format32bppArgb);
        using (var g = Graphics.FromImage(large))
        {
            g.CompositingMode = CompositingMode.SourceCopy;
            g.Clear(Color.Transparent);
            g.CompositingMode = CompositingMode.SourceOver;
            g.SmoothingMode = SmoothingMode.AntiAlias;
            g.PixelOffsetMode = PixelOffsetMode.HighQuality;
            g.ScaleTransform(Supersample, Supersample);
            draw(g);
        }
        using var final = new Bitmap(Size, Size, PixelFormat.Format32bppArgb);
        using (var g = Graphics.FromImage(final))
        {
            g.CompositingMode = CompositingMode.SourceCopy;
            g.Clear(Color.Transparent);
            g.InterpolationMode = InterpolationMode.HighQualityBicubic;
            g.PixelOffsetMode = PixelOffsetMode.HighQuality;
            g.DrawImage(large, new Rectangle(0, 0, Size, Size), new Rectangle(0, 0, Size * Supersample, Size * Supersample), GraphicsUnit.Pixel);
        }
        final.Save(path, ImageFormat.Png);
    }

    private static void FillShaded(Graphics g, IEnumerable<string> paths, string kind)
    {
        var top = kind.Contains("shadow") ? 172 : kind.Contains("highlight") ? 226 : 224;
        var bottom = kind.Contains("shadow") ? 219 : kind.Contains("highlight") ? 255 : 238;
        var colors = new ColorBlend
        {
            Positions = new[] { 0f, .48f, 1f },
            Colors = new[] { Gray(top), Gray(kind.Contains("shadow") ? 235 : 255), Gray(bottom) }
        };
        using var brush = new LinearGradientBrush(new PointF(256, 34), new PointF(256, 496), Color.White, Color.White);
        brush.InterpolationColors = colors;
        foreach (var data in paths) using (var path = MakePath(data)) g.FillPath(brush, path);
    }

    private static Color Gray(int value) => Color.FromArgb(255, value, value, value);

    private static void FillMany(Graphics g, IEnumerable<string> paths, Color color)
    {
        using var brush = new SolidBrush(color);
        foreach (var data in paths) using (var path = MakePath(data)) g.FillPath(brush, path);
    }

    private static GraphicsPath MakePath(string commands)
    {
        var path = new GraphicsPath(FillMode.Winding);
        var tokens = commands.Split(new[] { ' ', '\t', '\r', '\n' }, StringSplitOptions.RemoveEmptyEntries);
        var index = 0;
        PointF current = default;
        while (index < tokens.Length)
        {
            var op = tokens[index++];
            if (op == "M")
            {
                current = new PointF(Parse(tokens[index++]), Parse(tokens[index++]));
                path.StartFigure();
            }
            else if (op == "L")
            {
                var next = new PointF(Parse(tokens[index++]), Parse(tokens[index++]));
                path.AddLine(current, next);
                current = next;
            }
            else if (op == "C")
            {
                var c1 = new PointF(Parse(tokens[index++]), Parse(tokens[index++]));
                var c2 = new PointF(Parse(tokens[index++]), Parse(tokens[index++]));
                var end = new PointF(Parse(tokens[index++]), Parse(tokens[index++]));
                path.AddBezier(current, c1, c2, end);
                current = end;
            }
            else if (op == "Z") path.CloseFigure();
            else throw new InvalidDataException("Unknown authored path command: " + op);
        }
        return path;
    }

    private static float Parse(string value) => float.Parse(value, System.Globalization.CultureInfo.InvariantCulture);

    private static void Stroke(Graphics g, string data, Color color, float width, LineCap cap = LineCap.Round)
    {
        using var path = MakePath(data);
        using var pen = new Pen(color, width) { StartCap = cap, EndCap = cap, LineJoin = LineJoin.Round };
        g.DrawPath(pen, path);
    }

    private static string FaceSilhouette(string face)
    {
        var earLeft = face == "face_oval"
            ? "M 132 208 C 104 196 81 214 84 243 C 87 271 105 289 136 277 Z"
            : "M 132 210 C 105 199 84 216 87 242 C 90 268 108 285 136 275 Z";
        var earRight = face == "face_oval"
            ? "M 380 208 C 408 196 431 214 428 243 C 425 271 407 289 376 277 Z"
            : "M 380 210 C 407 199 428 216 425 242 C 422 268 404 285 376 275 Z";
        var head = face == "face_oval"
            ? "M 256 64 C 185 60 123 101 116 177 C 109 253 138 328 188 370 C 211 391 238 405 256 408 C 274 405 301 391 324 370 C 374 328 403 253 396 177 C 389 101 327 60 256 64 Z"
            : "M 256 64 C 185 60 123 101 116 177 C 109 245 132 299 169 338 C 193 371 226 398 256 410 C 286 398 319 371 343 338 C 380 299 403 245 396 177 C 389 101 327 60 256 64 Z";
        return earLeft + " " + earRight + " " + head;
    }

    private static string[] FaceShadows(string face)
    {
        return new[]
        {
            "M 125 180 C 133 145 156 125 185 115 C 169 148 164 178 166 211 C 168 244 178 270 193 293 C 171 282 151 258 140 229 C 129 212 124 195 125 180 Z",
            "M 387 180 C 379 145 356 125 327 115 C 343 148 348 178 346 211 C 344 244 334 270 319 293 C 341 282 361 258 372 229 C 383 212 388 195 387 180 Z"
        };
    }

    private static string[] FaceHighlights(string face)
    {
        return new[]
        {
            "M 226 133 C 235 124 247 120 256 121 C 265 120 277 124 286 133 C 276 140 267 144 256 145 C 245 144 236 140 226 133 Z",
            "M 177 263 C 186 255 198 253 208 258 C 205 266 198 271 188 275 C 182 272 178 268 177 263 Z",
            "M 335 263 C 326 255 314 253 304 258 C 307 266 314 271 324 275 C 330 272 334 268 335 263 Z"
        };
    }

    private static void DrawFaceInk(Graphics g, string face)
    {
        Stroke(g, FaceSilhouette(face), Ink, 3.2f);
        Stroke(g, "M 122 220 C 105 215 98 225 101 244 C 103 260 113 269 128 265", SoftInk, 2.2f);
        Stroke(g, "M 390 220 C 407 215 414 225 411 244 C 409 260 399 269 384 265", SoftInk, 2.2f);
        Stroke(g, "M 126 239 C 111 229 105 244 119 253", SoftInk, 2f);
        Stroke(g, "M 386 239 C 401 229 407 244 393 253", SoftInk, 2f);
        // Quiet, neutral mouth is fixed to the mouth anchor; nose and eyes remain modular.
        Stroke(g, "M 228 344 C 239 351 248 353 256 352 C 264 353 273 351 284 344", Ink, 3.1f);
        Stroke(g, "M 238 358 C 248 364 264 364 274 358", SoftInk, 1.7f);
    }

    private static string[] NoseShading(string face, string nose)
    {
        var offset = face == "face_tapered" ? 1 : 0;
        if (nose == "nose_straight")
            return new[]
            {
                $"M {252-offset} 267 C 251 281 245 297 241 307 C 238 315 245 321 256 322 C 267 321 274 315 271 307 C 267 297 261 281 260 267 Z",
                "M 241 307 C 235 313 235 318 244 320 C 249 321 253 319 256 317 C 251 313 247 309 241 307 Z",
                "M 271 307 C 277 313 277 318 268 320 C 263 321 259 319 256 317 C 261 313 265 309 271 307 Z"
            };
        return new[]
        {
            $"M {253-offset} 267 C 251 282 246 297 241 307 C 238 315 243 319 254 318 C 262 317 269 313 273 308 C 267 304 262 299 258 290 C 255 282 255 274 257 267 Z",
            "M 242 306 C 233 311 234 319 245 321 C 251 322 254 320 257 317 C 250 315 246 311 242 306 Z",
            "M 258 307 C 265 310 273 309 277 314 C 276 319 269 321 261 319 C 257 318 255 316 254 314 Z"
        };
    }

    private static string[] EyeWhites(string face, string eyes)
    {
        var eyeHeight = eyes == "eyes_open" ? 15 : 11;
        var yTop = 236 - eyeHeight;
        var yBot = 236 + eyeHeight;
        var left = $"M 171 236 C 184 {yTop} 208 {yTop} 222 236 C 208 {yBot} 184 {yBot} 171 236 Z";
        var right = $"M 290 236 C 304 {yTop} 328 {yTop} 341 236 C 328 {yBot} 304 {yBot} 290 236 Z";
        // Tapered face adapters close the outer corners slightly without shifting the eye line.
        if (face == "face_tapered")
            return new[] { left.Replace("171 236", "174 236"), right.Replace("341 236", "338 236") };
        return new[] { left, right };
    }

    private static string[] IrisShapes(string face, string eyes)
    {
        var radius = eyes == "eyes_open" ? 9 : 7;
        var top = 236 - radius;
        var bottom = 236 + radius;
        var xL = face == "face_tapered" ? 201 : 199;
        var xR = 512 - xL;
        return new[]
        {
            $"M {xL} {top} C {xL+6} {top} {xL+8} {bottom} {xL} {bottom} C {xL-8} {bottom} {xL-6} {top} {xL} {top} Z",
            $"M {xR} {top} C {xR+6} {top} {xR+8} {bottom} {xR} {bottom} C {xR-8} {bottom} {xR-6} {top} {xR} {top} Z"
        };
    }

    private static void DrawEyeInk(Graphics g, string face, string eyes)
    {
        var offset = face == "face_tapered" ? 2 : 0;
        var lTop = eyes == "eyes_open" ? 222 : 226;
        var lLow = eyes == "eyes_open" ? 250 : 247;
        Stroke(g, $"M {172+offset} 236 C 185 {lTop} 207 {lTop-2} 222 236 C 208 {lLow} 187 {lLow+2} {174+offset} 237", Ink, eyes == "eyes_lidded" ? 3.7f : 2.7f);
        Stroke(g, $"M 290 236 C 305 {lTop-2} 327 {lTop} {340-offset} 236 C {325-offset} {lLow+2} 304 {lLow} 290 236", Ink, eyes == "eyes_lidded" ? 3.7f : 2.7f);
        // Brows stay above the white shapes; the fringe leaves both eye assemblies visible.
        Stroke(g, "M 173 210 C 187 201 207 200 223 208", Ink, eyes == "eyes_lidded" ? 6f : 4.2f);
        Stroke(g, "M 289 208 C 305 200 325 201 339 210", Ink, eyes == "eyes_lidded" ? 6f : 4.2f);
        FillMany(g, new[]
        {
            "M 194 231 C 199 227 205 227 209 231 C 208 239 205 244 201 245 C 197 243 194 238 194 231 Z",
            "M 303 231 C 307 227 313 227 318 231 C 318 238 315 243 311 245 C 306 244 303 239 303 231 Z"
        }, Ink);
        FillMany(g, new[] { "M 198 229 C 200 227 203 228 204 230 C 202 232 199 232 198 229 Z", "M 307 229 C 309 227 312 228 313 230 C 311 232 308 232 307 229 Z" }, White);
        if (eyes == "eyes_lidded")
        {
            Stroke(g, "M 178 228 C 192 222 207 222 219 230", SoftInk, 2.3f);
            Stroke(g, "M 293 230 C 305 222 320 222 334 228", SoftInk, 2.3f);
        }
    }

    private static string[] HairShapes(string face, string hair, string side, string tone)
    {
        var isCoiled = hair == "hair_coiled";
        var front = side == "front";
        var inset = face == "face_tapered" ? 5 : 0;
        if (!isCoiled)
        {
            if (front)
                return new[] { $"M {132+inset} 197 C {128+inset} 152 {139+inset} 111 174 82 C 205 58 239 46 257 48 C 276 31 305 41 314 70 C 350 81 {384-inset} 119 {378-inset} 180 C 364 167 351 166 338 175 C 318 164 298 169 281 181 C 262 169 244 169 225 180 C 202 174 178 181 {132+inset} 197 Z" };
            return new[] { $"M {108+inset} 211 C {101+inset} 135 148 64 228 47 C 244 35 256 32 269 35 C 337 48 {407-inset} 116 {404-inset} 210 C {405-inset} 267 {397-inset} 301 {380-inset} 317 C 370 305 368 280 370 246 C 374 210 350 172 326 146 C 300 115 278 106 256 105 C 234 106 212 115 186 146 C 162 172 138 210 142 246 C 144 280 142 305 {132+inset} 317 C {115+inset} 301 {107+inset} 267 {108+inset} 211 Z" };
        }
        if (front)
        {
            return new[]
            {
                $"M {157+inset} 196 C {156+inset} 167 {163+inset} 151 {172+inset} 139 C {165+inset} 124 {171+inset} 105 187 99 C 185 83 197 70 214 73 C 213 54 231 43 246 53 C 252 35 273 34 282 51 C 296 39 317 48 316 66 C 336 63 349 78 344 96 C 361 105 {359-inset} 127 {347-inset} 141 C {358-inset} 155 {359-inset} 177 {354-inset} 197 C 340 185 327 181 314 186 C 302 175 287 177 273 188 C 259 174 244 175 227 188 C 211 177 188 181 {157+inset} 196 Z",
                "M 185 109 C 188 94 203 88 215 95 C 225 80 242 79 253 91 C 265 75 284 76 294 92 C 309 82 326 91 327 108 C 309 101 298 104 286 116 C 273 106 260 109 250 122 C 238 109 222 109 211 121 C 200 108 194 107 185 109 Z"
            };
        }
        return new[]
        {
            $"M {100+inset} 212 C {92+inset} 189 {103+inset} 173 {119+inset} 162 C {108+inset} 148 {113+inset} 129 {131+inset} 121 C {120+inset} 103 {132+inset} 82 158 81 C 163 61 193 47 216 57 C 225 35 250 32 260 52 C 273 30 296 33 303 57 C 335 44 361 58 354 80 C 382 78 {414-inset} 100 {397-inset} 119 C {415-inset} 131 {414-inset} 151 {397-inset} 164 C {411-inset} 177 {414-inset} 197 {403-inset} 215 C 380 199 355 193 328 197 C 310 182 291 182 272 196 C 257 180 240 181 220 196 C 200 182 179 184 {131+inset} 201 C {116+inset} 205 {108+inset} 210 {100+inset} 212 Z",
            "M 176 111 C 180 93 198 88 210 98 C 215 79 235 72 248 88 C 259 68 279 70 288 91 C 304 75 323 84 326 102 C 341 101 346 116 336 127 C 322 114 309 113 297 128 C 283 111 267 114 255 131 C 242 112 226 114 214 131 C 202 111 188 107 176 111 Z"
        };
    }

    private static void DrawHairInk(Graphics g, string face, string hair, string side)
    {
        var paths = HairShapes(face, hair, side, "base");
        var width = hair == "hair_coiled" ? 2.3f : 2.6f;
        Stroke(g, paths[0], Ink, width);
        if (side == "front")
        {
            if (hair == "hair_cropped")
            {
                Stroke(g, "M 171 169 C 191 160 208 164 221 174 M 222 173 C 238 159 253 159 270 176 M 271 175 C 286 160 303 160 318 174 M 319 171 C 330 164 341 166 351 174", SoftInk, 2.2f);
                Stroke(g, "M 202 103 C 218 84 236 74 253 72 M 286 72 C 307 75 326 87 338 105", Color.FromArgb(140, 249, 208, 144), 2.5f);
            }
            else
            {
                Stroke(g, "M 182 111 C 191 97 203 93 214 99 M 217 93 C 226 77 242 76 251 91 M 258 87 C 267 72 283 77 290 94 M 298 96 C 309 84 325 90 328 105", SoftInk, 2.2f);
                foreach (var curl in new[]
                {
                    "M 191 140 C 185 130 196 122 204 128 C 211 134 205 145 197 143",
                    "M 226 133 C 218 123 229 115 237 121 C 244 127 239 138 231 136",
                    "M 262 133 C 254 123 265 115 273 121 C 280 127 275 138 267 136",
                    "M 296 140 C 288 130 299 122 307 128 C 314 134 309 145 301 143"
                }) Stroke(g, curl, Color.FromArgb(180, 245, 202, 137), 2f);
            }
        }
        else if (hair == "hair_coiled")
        {
            Stroke(g, "M 159 188 C 151 174 158 161 169 160 M 348 188 C 356 174 349 161 338 160 M 164 261 C 155 250 159 235 169 231 M 347 261 C 357 250 353 235 343 231", SoftInk, 2.2f);
        }
        else
        {
            Stroke(g, "M 159 184 C 156 159 166 137 179 120 M 180 282 C 168 269 167 251 169 237 M 353 184 C 356 159 346 137 333 120 M 332 282 C 344 269 345 251 343 237", SoftInk, 2.2f);
        }
    }

    private static void WriteSupporting()
    {
        var neck = Out("assets/portraits/supporting/neck");
        var neckShape = "M 219 374 C 222 391 213 408 198 422 C 184 436 164 446 145 456 C 179 475 216 485 256 486 C 296 485 333 475 367 456 C 348 446 328 436 314 422 C 299 408 290 391 293 374 C 281 388 268 397 256 400 C 244 397 231 388 219 374 Z";
        SavePair(neck, "base", new[] { neckShape }, new[] { neckShape }, "face-base");
        SavePair(neck, "shadow", new[] { "M 220 397 C 234 414 246 420 256 421 C 266 420 278 414 292 397 L 302 440 C 286 452 271 457 256 457 C 241 457 226 452 210 440 Z" }, new[] { "M 220 397 C 234 414 246 420 256 421 C 266 420 278 414 292 397 L 302 440 C 286 452 271 457 256 457 C 241 457 226 452 210 440 Z" }, "face-shadow");
        SavePair(neck, "highlight", new[] { "M 239 405 C 244 411 250 414 256 415 C 262 414 268 411 273 405 L 268 444 C 264 447 260 449 256 449 C 252 449 248 447 244 444 Z" }, new[] { "M 239 405 C 244 411 250 414 256 415 C 262 414 268 411 273 405 L 268 444 C 264 447 260 449 256 449 C 252 449 248 447 244 444 Z" }, "face-highlight");
        SaveInk(neck, "ink.png", g => Stroke(g, "M 219 391 C 230 411 244 426 256 428 C 268 426 282 411 293 391", SoftInk, 2.3f));

        var shoulders = Out("assets/portraits/supporting/shoulders");
        var shoulderShape = "M 211 427 C 196 439 170 447 141 454 C 105 462 67 475 44 495 L 468 495 C 445 475 407 462 371 454 C 342 447 316 439 301 427 C 290 446 276 455 256 459 C 236 455 222 446 211 427 Z";
        SavePair(shoulders, "base", new[] { shoulderShape }, new[] { shoulderShape }, "shoulders-base");
        SavePair(shoulders, "shadow", new[] { "M 118 481 C 166 463 202 458 226 454 C 239 466 247 471 256 473 C 265 471 273 466 286 454 C 310 458 346 463 394 481 L 421 495 L 91 495 Z" }, new[] { "M 118 481 C 166 463 202 458 226 454 C 239 466 247 471 256 473 C 265 471 273 466 286 454 C 310 458 346 463 394 481 L 421 495 L 91 495 Z" }, "shoulders-shadow");
        SavePair(shoulders, "highlight", new[] { "M 141 466 C 183 452 211 447 225 445 C 235 455 245 461 256 463 C 267 461 277 455 287 445 C 301 447 329 452 371 466 C 329 461 297 463 282 472 C 271 478 263 481 256 481 C 249 481 241 478 230 472 C 215 463 183 461 141 466 Z" }, new[] { "M 141 466 C 183 452 211 447 225 445 C 235 455 245 461 256 463 C 267 461 277 455 287 445 C 301 447 329 452 371 466 C 329 461 297 463 282 472 C 271 478 263 481 256 481 C 249 481 241 478 230 472 C 215 463 183 461 141 466 Z" }, "shoulders-highlight");
        SaveInk(shoulders, "ink.png", g =>
        {
            Stroke(g, "M 211 430 C 225 449 240 458 256 461 C 272 458 287 449 301 430", Ink, 3f);
            Stroke(g, "M 210 443 C 223 455 237 463 248 467 M 302 443 C 289 455 275 463 264 467", SoftInk, 2.2f);
            Stroke(g, "M 105 489 C 151 471 191 464 218 465 M 294 465 C 321 464 361 471 407 489", Color.FromArgb(185, 182, 198, 196), 2.4f);
        });
        WriteFallbacks();
    }

    private static void WriteFallbacks()
    {
        var folder = Out("assets/portraits/supporting");
        SaveRaster(Path.Combine(folder, "adult_fallback.png"), g =>
        {
            FillMany(g, new[] { "M 186 488 C 193 450 218 433 238 425 L 238 406 C 218 393 205 374 201 346 C 181 339 176 318 187 303 C 182 260 182 214 196 179 C 211 141 235 127 256 127 C 277 127 301 141 316 179 C 330 214 330 260 325 303 C 336 318 331 339 311 346 C 307 374 294 393 274 406 L 274 425 C 294 433 319 450 326 488 Z" }, Color.FromArgb(255, 140, 106, 81));
            FillMany(g, new[] { "M 192 488 C 196 454 218 439 241 432 C 251 437 261 437 271 432 C 294 439 316 454 320 488 Z" }, Color.FromArgb(255, 68, 82, 92));
            Stroke(g, "M 205 306 C 209 332 223 350 239 362 C 248 369 263 369 273 362 C 289 350 303 332 307 306", Ink, 4f);
            Stroke(g, "M 212 270 C 225 262 239 262 250 270 M 262 270 C 273 262 287 262 300 270", Ink, 4f);
            FillMany(g, new[] { "M 228 277 C 233 277 236 286 232 291 C 228 295 224 290 224 284 Z", "M 281 277 C 286 277 290 284 286 291 C 282 295 278 290 278 284 Z" }, Ink);
            Stroke(g, "M 251 280 C 250 297 245 311 244 317 C 248 322 258 322 264 318", Ink, 3f);
            Stroke(g, "M 237 341 C 248 346 264 346 275 341", Ink, 3f);
            Stroke(g, "M 180 246 C 172 204 188 148 220 123 C 238 110 278 109 296 126 C 327 154 338 201 326 249", Ink, 4f);
        });
        SaveRaster(Path.Combine(folder, "child_silhouette.png"), g =>
        {
            FillMany(g, new[] { "M 202 488 C 205 455 221 439 238 433 L 238 414 C 217 400 204 378 202 349 C 185 344 178 326 186 312 C 182 269 184 228 199 197 C 211 170 233 158 256 158 C 279 158 301 170 313 197 C 328 228 330 269 326 312 C 334 326 327 344 310 349 C 308 378 295 400 274 414 L 274 433 C 291 439 307 455 310 488 Z" }, Color.FromArgb(255, 168, 132, 104));
            FillMany(g, new[] { "M 208 488 C 211 459 227 446 242 440 C 252 445 260 445 270 440 C 285 446 301 459 304 488 Z" }, Color.FromArgb(255, 84, 99, 104));
            Stroke(g, "M 210 283 C 224 274 239 274 250 282 M 262 282 C 273 274 288 274 302 283", Ink, 4f);
            FillMany(g, new[] { "M 229 290 C 234 290 237 299 233 304 C 229 308 225 303 225 297 Z", "M 280 290 C 285 290 289 297 285 304 C 281 308 277 303 277 297 Z" }, Ink);
            Stroke(g, "M 251 293 C 251 308 247 321 246 326 C 251 330 261 330 266 326", Ink, 3f);
            Stroke(g, "M 240 350 C 249 354 263 354 272 350", Ink, 2.8f);
            Stroke(g, "M 185 260 C 178 219 197 180 224 160 C 245 145 268 146 288 159 C 316 177 333 220 327 260", Ink, 4f);
        });
        SaveRaster(Path.Combine(folder, "unknown_silhouette.png"), g =>
        {
            FillMany(g, new[] { "M 186 488 C 193 450 218 433 238 425 L 238 406 C 218 393 205 374 201 346 C 181 339 176 318 187 303 C 182 260 182 214 196 179 C 211 141 235 127 256 127 C 277 127 301 141 316 179 C 330 214 330 260 325 303 C 336 318 331 339 311 346 C 307 374 294 393 274 406 L 274 425 C 294 433 319 450 326 488 Z" }, Color.FromArgb(255, 89, 101, 107));
            FillMany(g, new[] { "M 212 270 C 219 252 237 242 256 242 C 275 242 293 252 300 270 C 291 288 276 298 256 298 C 236 298 221 288 212 270 Z" }, Color.FromArgb(255, 189, 197, 191));
            Stroke(g, "M 212 343 C 226 319 243 309 256 309 C 269 309 286 319 300 343", Color.FromArgb(255, 89, 101, 107), 5f);
            Stroke(g, "M 197 190 C 215 146 238 133 256 133 C 274 133 297 146 315 190", Color.FromArgb(255, 36, 44, 49), 6f);
        });
    }

    private static Ramp ReadRamp(JsonElement palettes, string id)
    {
        foreach (var item in palettes.EnumerateArray())
        {
            if (item.GetProperty("id").GetString() != id) continue;
            return new Ramp(id, ReadRgb(item.GetProperty("shadow")), ReadRgb(item.GetProperty("base")), ReadRgb(item.GetProperty("highlight")));
        }
        throw new InvalidDataException("Missing palette: " + id);
    }

    private static byte[] ReadRgb(JsonElement value) => value.EnumerateArray().Select(channel => (byte)channel.GetInt32()).ToArray();

    private static void ValidateEveryTupleAndPalette(JsonElement catalog)
    {
        if(catalog.GetProperty("catalog_revision").GetInt32()==2)
        {
            ValidateReleaseMatrix(catalog);
            return;
        }
        var skins = catalog.GetProperty("skin_palettes").EnumerateArray().Select(item => ReadRamp(catalog.GetProperty("skin_palettes"), item.GetProperty("id").GetString()!)).ToArray();
        var hairRamps = catalog.GetProperty("hair_palettes").EnumerateArray().Select(item => ReadRamp(catalog.GetProperty("hair_palettes"), item.GetProperty("id").GetString()!)).ToArray();
        var irisRamps = catalog.GetProperty("eye_palettes").EnumerateArray().Select(item => ReadRamp(catalog.GetProperty("eye_palettes"), item.GetProperty("id").GetString()!)).ToArray();
        int tupleCount = 0, legalColorCount = 0;
        foreach (var tuple in catalog.GetProperty("geometry").EnumerateArray())
        {
            var face = tuple.GetProperty("face_id").GetString()!;
            var nose = tuple.GetProperty("nose_id").GetString()!;
            var eyes = tuple.GetProperty("eyes_id").GetString()!;
            var hair = tuple.GetProperty("hair_id").GetString()!;
            var hairChoices = hair == "hair_bald" ? hairRamps.Where(value => value.Id == "none").ToArray() : hairRamps.Where(value => value.Id != "none").ToArray();
            foreach (var selectedSkin in skins)
            foreach (var selectedHair in hairChoices)
            foreach (var selectedIris in irisRamps)
            {
                var pixels = CompositePixels(face, nose, eyes, hair, selectedSkin, selectedHair, selectedIris);
                AssertCanvas(pixels, $"{face}/{nose}/{eyes}/{hair}/{selectedSkin.Id}/{selectedHair.Id}/{selectedIris.Id}", legalColorCount == 0);
                legalColorCount++;
            }
            tupleCount++;
        }
        if (tupleCount != 24 || legalColorCount != 336)
            throw new InvalidDataException($"Expected 24 geometry tuples and 336 legal color combinations; saw {tupleCount} and {legalColorCount}.");
        if (skins.Length != 3 || hairRamps.Length != 4 || irisRamps.Length != 2)
            throw new InvalidDataException("Palette-edge check expected 3 skin, 3 chromatic hair, and 2 iris ramps.");
        Console.WriteLine($"Pixel matrix passed: {tupleCount} geometry tuples and all {legalColorCount} legal palette combinations fully composited; every palette edge is represented in the contact sheet.");
    }

    private static void AssertCanvas(byte[] pixels, string label, bool strictBounds)
    {
        int count = 0, minX = Size, minY = Size, maxX = -1, maxY = -1;
        for (int y = 0; y < Size; y++)
        for (int x = 0; x < Size; x++)
        {
            if (pixels[(y * Size + x) * 4 + 3] == 0) continue;
            count++;
            minX = Math.Min(minX, x); minY = Math.Min(minY, y);
            maxX = Math.Max(maxX, x); maxY = Math.Max(maxY, y);
        }
        if (count < 10000) throw new InvalidDataException($"{label} rendered too little visible portrait art ({count} pixels).");
        if (minX < 40 || minY < 24 || maxX >= 472 || maxY >= 496)
            throw new InvalidDataException($"{label} escaped the catalog safe rectangle: [{minX},{minY},{maxX},{maxY}].");
        if (strictBounds && (minY > 65 || maxY < 480 || minX > 47 || maxX < 465))
            throw new InvalidDataException("The proof portrait does not use the frozen bust rig's full intended span.");
    }

    private static byte[] CompositePixels(string face, string nose, string eyes, string hair, Ramp skin, Ramp hairPalette, Ramp iris)
    {
        var output = new byte[Size * Size * 4];
        var baseDir = Out($"assets/portraits/faces/{face}");
        if (hair != "hair_bald")
        {
            var hairDir = Out($"assets/portraits/hair/{face}/{hair}");
            DrawShaded(output, hairDir, "rear_", hairPalette);
        }
        DrawShaded(output, Out("assets/portraits/supporting/shoulders"), "", NeutralShoulderRamp());
        DrawShaded(output, Out("assets/portraits/supporting/neck"), "", skin);
        DrawShaded(output, baseDir, "", skin, false);
        var eyeDir = Out($"assets/portraits/eyes/{face}");
        OverFile(output, Path.Combine(eyeDir, eyes + "_whites.png"));
        TintFiles(output, Path.Combine(eyeDir, eyes + "_iris_source.png"), Path.Combine(eyeDir, eyes + "_iris_mask.png"), iris.Base);
        OverFile(output, Path.Combine(eyeDir, eyes + "_ink.png"));
        var noseDir = Out($"assets/portraits/noses/{face}");
        TintFiles(output, Path.Combine(noseDir, nose + "_source.png"), Path.Combine(noseDir, nose + "_mask.png"), skin.Base);
        OverFile(output, Path.Combine(baseDir, "ink.png"));
        if (hair != "hair_bald")
        {
            var hairDir = Out($"assets/portraits/hair/{face}/{hair}");
            DrawShaded(output, hairDir, "front_", hairPalette);
        }
        return output;
    }

    private static Bitmap Composite(string face, string nose, string eyes, string hair, Ramp skin, Ramp hairPalette, Ramp iris)
    {
        var bitmap = new Bitmap(Size, Size, PixelFormat.Format32bppArgb);
        WritePixels(bitmap, CompositePixels(face, nose, eyes, hair, skin, hairPalette, iris));
        return bitmap;
    }

    private static Ramp NeutralShoulderRamp() => new Ramp("neutral-cloth", new byte[] { 39, 51, 58 }, new byte[] { 84, 101, 108 }, new byte[] { 153, 164, 157 });

    private static void DrawShaded(byte[] output, string folder, string prefix, Ramp ramp, bool includeInk = true)
    {
        TintFiles(output, Path.Combine(folder, prefix + "base_source.png"), Path.Combine(folder, prefix + "base_mask.png"), ramp.Base);
        var shadowSource = Path.Combine(folder, prefix + "shadow_source.png");
        if (File.Exists(shadowSource)) TintFiles(output, shadowSource, Path.Combine(folder, prefix + "shadow_mask.png"), ramp.Shadow);
        var highlightSource = Path.Combine(folder, prefix + "highlight_source.png");
        if (File.Exists(highlightSource)) TintFiles(output, highlightSource, Path.Combine(folder, prefix + "highlight_mask.png"), ramp.Highlight);
        var ink = Path.Combine(folder, prefix + "ink.png");
        if (includeInk && File.Exists(ink)) OverFile(output, ink);
    }

    private static void TintFiles(byte[] destination, string sourcePath, string maskPath, byte[] tint)
    {
        var source = ReadPngPixels(sourcePath);
        var mask = ReadPngPixels(maskPath);
        for (int offset = 0; offset < source.Length; offset += 4)
        {
            var coverage = (mask[offset + 2] / 255.0) * (mask[offset + 3] / 255.0);
            var alpha = Byte(source[offset + 3] / 255.0 * coverage);
            var red = Byte(source[offset + 2] / 255.0 * tint[0] / 255.0);
            var green = Byte(source[offset + 2] / 255.0 * tint[1] / 255.0);
            var blue = Byte(source[offset + 2] / 255.0 * tint[2] / 255.0);
            BlendPixel(destination, offset, red, green, blue, alpha);
        }
    }

    private static byte Byte(double value) => (byte)Math.Clamp((int)Math.Round(Math.Clamp(value, 0, 1) * 255.0, MidpointRounding.AwayFromZero), 0, 255);

    private static byte[] ReadPngPixels(string path)
    {
        if (PixelCache.TryGetValue(path, out var cached)) return cached;
        using var original = new Bitmap(path);
        if (original.Width != Size || original.Height != Size)
            throw new InvalidDataException("Portrait layer is not 512x512: " + path);
        using var rgba = original.Clone(new Rectangle(0, 0, original.Width, original.Height), PixelFormat.Format32bppArgb);
        var pixels = ReadPixels(rgba);
        PixelCache[path] = pixels;
        return pixels;
    }

    private static void OverFile(byte[] destination, string path)
    {
        AlphaOver(destination, ReadPngPixels(path));
    }

    private static void AlphaOver(byte[] destination, byte[] source)
    {
        for (int offset = 0; offset < source.Length; offset += 4)
        {
            BlendPixel(destination, offset, source[offset + 2], source[offset + 1], source[offset], source[offset + 3]);
        }
    }

    private static void BlendPixel(byte[] destination, int offset, byte red, byte green, byte blue, byte alpha)
    {
        if (alpha == 0) return;
        var sa = alpha / 255.0; var da = destination[offset + 3] / 255.0;
        var oa = sa + da * (1.0 - sa);
        if (oa <= 0)
        {
            destination[offset] = destination[offset + 1] = destination[offset + 2] = destination[offset + 3] = 0;
            return;
        }
        destination[offset + 2] = Byte(((red * sa + destination[offset + 2] * da * (1.0 - sa)) / oa) / 255.0);
        destination[offset + 1] = Byte(((green * sa + destination[offset + 1] * da * (1.0 - sa)) / oa) / 255.0);
        destination[offset] = Byte(((blue * sa + destination[offset] * da * (1.0 - sa)) / oa) / 255.0);
        destination[offset + 3] = Byte(oa);
    }

    private static byte[] ReadPixels(Bitmap bitmap)
    {
        var data = bitmap.LockBits(new Rectangle(0, 0, bitmap.Width, bitmap.Height), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
        try
        {
            var rowBytes = bitmap.Width * 4;
            var pixels = new byte[rowBytes * bitmap.Height];
            for (int y = 0; y < bitmap.Height; y++) Marshal.Copy(IntPtr.Add(data.Scan0, y * data.Stride), pixels, y * rowBytes, rowBytes);
            return pixels;
        }
        finally { bitmap.UnlockBits(data); }
    }

    private static void WritePixels(Bitmap bitmap, byte[] pixels)
    {
        var data = bitmap.LockBits(new Rectangle(0, 0, bitmap.Width, bitmap.Height), ImageLockMode.WriteOnly, PixelFormat.Format32bppArgb);
        try
        {
            var rowBytes = bitmap.Width * 4;
            for (int y = 0; y < bitmap.Height; y++) Marshal.Copy(pixels, y * rowBytes, IntPtr.Add(data.Scan0, y * data.Stride), rowBytes);
        }
        finally { bitmap.UnlockBits(data); }
    }

    private static void GenerateContactSheet(JsonElement catalog)
    {
        if(catalog.GetProperty("catalog_revision").GetInt32()==2)
        {
            GenerateReleaseContactSheets(catalog);
            return;
        }
        var samples = new[]
        {
            new Sample("Oval · straight · open · bald / fair · hazel", "face_oval", "nose_straight", "eyes_open", "hair_bald", "skin_fair", "none", "eyes_hazel", false),
            new Sample("Oval · straight · open · cropped / umber · ebony", "face_oval", "nose_straight", "eyes_open", "hair_cropped", "skin_umber", "hair_ebony", "eyes_brown", true),
            new Sample("Oval · upturned · lidded · coiled / ochre · copper", "face_oval", "nose_upturned", "eyes_lidded", "hair_coiled", "skin_ochre", "hair_copper", "eyes_hazel", false),
            new Sample("Oval · upturned · open · cropped / fair · flax", "face_oval", "nose_upturned", "eyes_open", "hair_cropped", "skin_fair", "hair_flax", "eyes_brown", true),
            new Sample("Tapered · straight · open · bald / umber · hazel", "face_tapered", "nose_straight", "eyes_open", "hair_bald", "skin_umber", "none", "eyes_hazel", false),
            new Sample("Tapered · straight · open · cropped / fair · flax", "face_tapered", "nose_straight", "eyes_open", "hair_cropped", "skin_fair", "hair_flax", "eyes_brown", true),
            new Sample("Tapered · straight · lidded · coiled / ochre · ebony", "face_tapered", "nose_straight", "eyes_lidded", "hair_coiled", "skin_ochre", "hair_ebony", "eyes_hazel", false),
            new Sample("Tapered · upturned · open · cropped / ochre · copper", "face_tapered", "nose_upturned", "eyes_open", "hair_cropped", "skin_ochre", "hair_copper", "eyes_brown", true),
            new Sample("Tapered · upturned · lidded · coiled / umber · flax", "face_tapered", "nose_upturned", "eyes_lidded", "hair_coiled", "skin_umber", "hair_flax", "eyes_hazel", false),
            // The last two are a deliberately adjacent near-pair: only jaw silhouette changes.
            new Sample("Near pair A · same features + colors · oval", "face_oval", "nose_straight", "eyes_open", "hair_cropped", "skin_ochre", "hair_copper", "eyes_brown", false),
            new Sample("Near pair B · same features + colors · tapered", "face_tapered", "nose_straight", "eyes_open", "hair_cropped", "skin_ochre", "hair_copper", "eyes_brown", true),
            new Sample("Palette edges · umber · flax · hazel", "face_oval", "nose_upturned", "eyes_lidded", "hair_coiled", "skin_umber", "hair_flax", "eyes_hazel", false)
        };
        var skins = catalog.GetProperty("skin_palettes");
        var hairs = catalog.GetProperty("hair_palettes");
        var eyes = catalog.GetProperty("eye_palettes");
        const int labelWidth = 340, groupWidth = 180, rowHeight = 145, top = 74;
        var sheet = new Bitmap(labelWidth + 3 * groupWidth, top + samples.Length * rowHeight, PixelFormat.Format32bppArgb);
        using (var g = Graphics.FromImage(sheet))
        {
            g.Clear(Color.FromArgb(255, 244, 239, 229));
            g.SmoothingMode = SmoothingMode.AntiAlias;
            using var titleFont = new Font("Georgia", 25, FontStyle.Bold, GraphicsUnit.Pixel);
            using var headFont = new Font("Segoe UI", 15, FontStyle.Bold, GraphicsUnit.Pixel);
            using var labelFont = new Font("Segoe UI", 12, FontStyle.Regular, GraphicsUnit.Pixel);
            using var titleBrush = new SolidBrush(Color.FromArgb(255, 46, 40, 38));
            g.DrawString("KESTRUM · G02 PORTRAIT ART PROOF", titleFont, titleBrush, 24, 15);
            for (int col = 0; col < 3; col++)
                g.DrawString(new[] { "40 px", "64 px", "128 px" }[col], headFont, titleBrush, labelWidth + col * groupWidth + 8, 48);
            using var labelBrush = new SolidBrush(Color.FromArgb(255, 60, 53, 47));
            using var light = new SolidBrush(Color.FromArgb(255, 237, 225, 205));
            using var dark = new SolidBrush(Color.FromArgb(255, 42, 47, 54));
            using var pen = new Pen(Color.FromArgb(255, 210, 199, 182), 1);
            for (int i = 0; i < samples.Length; i++)
            {
                var sample = samples[i];
                int y = top + i * rowHeight;
                g.DrawString(sample.Label, labelFont, labelBrush, new RectangleF(24, y + 30, labelWidth - 35, 80));
                var image = Composite(sample.Face, sample.Nose, sample.Eyes, sample.Hair,
                    ReadRamp(skins, sample.Skin), ReadRamp(hairs, sample.HairPalette), ReadRamp(eyes, sample.Iris));
                try
                {
                    for (int col = 0; col < 3; col++)
                    {
                        var background = sample.DarkBackground ? dark : light;
                        var x = labelWidth + col * groupWidth + 10;
                        var box = new Rectangle(x, y + 5, groupWidth - 20, rowHeight - 10);
                        g.FillRectangle(background, box);
                        int size = new[] { 40, 64, 128 }[col];
                        int px = box.X + (box.Width - size) / 2;
                        int py = box.Y + (box.Height - size) / 2;
                        using var reduced = DownsampleArea(image, size, size);
                        g.DrawImageUnscaled(reduced, px, py);
                        g.DrawRectangle(pen, box);
                    }
                }
                finally { image.Dispose(); }
            }
        }
        var output = Out("docs/verification/portrait_contact_sheet.png");
        Directory.CreateDirectory(Path.GetDirectoryName(output)!);
        sheet.Save(output, ImageFormat.Png);
        sheet.Dispose();
    }

    private static Bitmap DownsampleArea(Bitmap input, int width, int height)
    {
        var source = ReadPixels(input);
        var reduced = new byte[width * height * 4];
        var output = new Bitmap(width, height, PixelFormat.Format32bppArgb);
        for (int y = 0; y < height; y++)
        {
            double top = y * input.Height / (double)height;
            double bottom = (y + 1) * input.Height / (double)height;
            int y0 = (int)Math.Floor(top), y1 = (int)Math.Ceiling(bottom);
            for (int x = 0; x < width; x++)
            {
                double left = x * input.Width / (double)width;
                double right = (x + 1) * input.Width / (double)width;
                int x0 = (int)Math.Floor(left), x1 = (int)Math.Ceiling(right);
                double total = 0, alpha = 0, red = 0, green = 0, blue = 0;
                for (int sy = y0; sy < y1; sy++)
                for (int sx = x0; sx < x1; sx++)
                {
                    int offset = (sy * input.Width + sx) * 4;
                    double overlap = Math.Max(0, Math.Min(bottom, sy + 1) - Math.Max(top, sy)) * Math.Max(0, Math.Min(right, sx + 1) - Math.Max(left, sx));
                    double a = source[offset + 3] / 255.0;
                    total += overlap; alpha += a * overlap;
                    red += source[offset + 2] / 255.0 * a * overlap;
                    green += source[offset + 1] / 255.0 * a * overlap;
                    blue += source[offset] / 255.0 * a * overlap;
                }
                var aOut = alpha / total;
                int target = (y * width + x) * 4;
                if (aOut > 0)
                {
                    reduced[target] = Byte(blue / alpha);
                    reduced[target + 1] = Byte(green / alpha);
                    reduced[target + 2] = Byte(red / alpha);
                    reduced[target + 3] = Byte(aOut);
                }
            }
        }
        WritePixels(output, reduced);
        return output;
    }

    private static void ValidateExportSet()
    {
        var pngs = Directory.GetFiles(Out("assets/portraits"), "*.png", SearchOption.AllDirectories);
        if (pngs.Length !=111 && pngs.Length!=509) throw new InvalidDataException($"Expected 111 G02 or 509 release exports, found {pngs.Length} PNGs.");
        int pairCount = 0;
        foreach (var path in pngs)
        {
            using var bitmap = new Bitmap(path);
            if (bitmap.Width != Size || bitmap.Height != Size || bitmap.PixelFormat != PixelFormat.Format32bppArgb)
                throw new InvalidDataException("Every proof PNG must be aligned 512x512 RGBA: " + path);
            if (!HasVisibleAlpha(ReadPixels(bitmap))) throw new InvalidDataException("Empty PNG in authored proof: " + path);
            if (!Path.GetFileName(path).EndsWith("_source.png", StringComparison.Ordinal)) continue;
            var maskPath = Path.Combine(Path.GetDirectoryName(path)!, Path.GetFileName(path).Replace("_source.png", "_mask.png", StringComparison.Ordinal));
            if (!File.Exists(maskPath)) throw new InvalidDataException("Missing aligned mask for source layer: " + path);
            var sourcePixels = ReadPngPixels(path);
            var maskPixels = ReadPngPixels(maskPath);
            for (int offset = 0; offset < sourcePixels.Length; offset += 4)
            {
                var sourceAlpha = sourcePixels[offset + 3];
                var maskAlpha = maskPixels[offset + 3];
                if (maskAlpha > 0 && sourceAlpha == 0)
                    throw new InvalidDataException("Mask extends beyond its authored source: " + maskPath);
                if (sourceAlpha >= 8 && (maskAlpha == 0 || maskPixels[offset + 2] == 0))
                    throw new InvalidDataException("Source and mask coverage do not align: " + path);
            }
            pairCount++;
        }
        int expectedPairs=pngs.Length==509?210:44;
        if (pairCount != expectedPairs) throw new InvalidDataException($"Expected {expectedPairs} aligned source/mask pairs, found {pairCount}.");
        Console.WriteLine($"Export check passed: {pngs.Length} aligned, nonempty RGBA PNG layers and fallbacks; all {pairCount} source/mask pairs align.");
    }

    private static bool HasVisibleAlpha(byte[] pixels)
    {
        for (int offset = 3; offset < pixels.Length; offset += 4) if (pixels[offset] != 0) return true;
        return false;
    }
}
