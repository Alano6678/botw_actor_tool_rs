# Generates assets/app.ico (dark rounded square + golden Triforce) for the exe icon.
Add-Type -AssemblyName System.Drawing

$dir = "D:\botw_actor_tool-master\botw_actor_tool_rs\assets"
New-Item -ItemType Directory -Force -Path $dir | Out-Null

$sizes = 256, 48, 32, 16
$pngs = @()

foreach ($sz in $sizes) {
    $bmp = [System.Drawing.Bitmap]::new($sz, $sz)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.Clear([System.Drawing.Color]::Transparent)

    $bgColor = [System.Drawing.Color]::FromArgb(255, 28, 31, 46)
    $bg = [System.Drawing.SolidBrush]::new($bgColor)
    $rr = [single]($sz * 0.12)
    $path = [System.Drawing.Drawing2D.GraphicsPath]::new()
    $path.AddArc(0, 0, $rr * 2, $rr * 2, 180, 90)
    $path.AddArc($sz - $rr * 2, 0, $rr * 2, $rr * 2, 270, 90)
    $path.AddArc($sz - $rr * 2, $sz - $rr * 2, $rr * 2, $rr * 2, 0, 90)
    $path.AddArc(0, $sz - $rr * 2, $rr * 2, $rr * 2, 90, 90)
    $path.CloseFigure()
    $g.FillPath($bg, $path)

    $gold = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(255, 255, 199, 57))
    $pts1 = [System.Drawing.PointF[]]@(
        [System.Drawing.PointF]::new([single]($sz * 0.50), [single]($sz * 0.16)),
        [System.Drawing.PointF]::new([single]($sz * 0.87), [single]($sz * 0.74)),
        [System.Drawing.PointF]::new([single]($sz * 0.13), [single]($sz * 0.74))
    )
    $g.FillPolygon($gold, $pts1)
    $pts2 = [System.Drawing.PointF[]]@(
        [System.Drawing.PointF]::new([single]($sz * 0.50), [single]($sz * 0.44)),
        [System.Drawing.PointF]::new([single]($sz * 0.73), [single]($sz * 0.74)),
        [System.Drawing.PointF]::new([single]($sz * 0.27), [single]($sz * 0.74))
    )
    $g.FillPolygon($bg, $pts2)
    $g.Dispose()

    $ms = [System.IO.MemoryStream]::new()
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $pngs += , $ms.ToArray()
    $ms.Dispose()
    $bmp.Dispose()
}

# Assemble a PNG-compressed ICO with one entry per size.
$stream = [System.IO.MemoryStream]::new()
$bw = [System.IO.BinaryWriter]::new($stream)
$bw.Write([uint16]0)          # reserved
$bw.Write([uint16]1)          # type: icon
$bw.Write([uint16]$sizes.Count)
$offset = 6 + 16 * $sizes.Count
for ($i = 0; $i -lt $sizes.Count; $i++) {
    $sz = $sizes[$i]
    $w = if ($sz -ge 256) { 0 } else { $sz }
    $h = if ($sz -ge 256) { 0 } else { $sz }
    $bw.Write([byte]$w)
    $bw.Write([byte]$h)
    $bw.Write([byte]0)
    $bw.Write([byte]0)
    $bw.Write([uint16]1)      # color planes
    $bw.Write([uint16]32)     # bpp
    $bw.Write([uint32]$pngs[$i].Length)
    $bw.Write([uint32]$offset)
    $offset += $pngs[$i].Length
}
foreach ($p in $pngs) { $bw.Write($p) }
$bw.Flush()
[System.IO.File]::WriteAllBytes("$dir\app.ico", $stream.ToArray())
$bw.Dispose()
$stream.Dispose()

Write-Output ("ico size: " + (Get-Item "$dir\app.ico").Length + " bytes")
