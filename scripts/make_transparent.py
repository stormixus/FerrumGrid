import os
from PIL import Image, ImageDraw, ImageChops

def cubic_bezier(p0, p1, p2, p3, steps=30):
    points = []
    for i in range(steps + 1):
        t = i / steps
        x = (1-t)**3 * p0[0] + 3*(1-t)**2 * t * p1[0] + 3*(1-t) * t**2 * p2[0] + t**3 * p3[0]
        y = (1-t)**3 * p0[1] + 3*(1-t)**2 * t * p1[1] + 3*(1-t) * t**2 * p2[1] + t**3 * p3[1]
        points.append((x, y))
    return points

def generate_squircle_mask(size=1024, scale_factor=4):
    """
    Generates a mathematically perfect, anti-aliased macOS squircle mask.
    Super-samples the vector path definition at high resolution and scales
    it down using high-quality Lanczos resampling to achieve flawless edge anti-aliasing.
    """
    large_size = size * scale_factor
    mask = Image.new("L", (large_size, large_size), 0)
    draw = ImageDraw.Draw(mask)
    
    # 1. Base squircle path from the official vector assets/app-icon-dark.svg
    vertices = []
    
    # M174.22 0 L850.754 0
    vertices.append((174.22, 0))
    vertices.append((850.754, 0))
    
    # C864.977 2.39104 875.796 4.29095 889.763 8.59805
    vertices.extend(cubic_bezier((850.754, 0), (864.977, 2.39104), (875.796, 4.29095), (889.763, 8.59805)))
    # C930.553 21.2544 965.987 47.0971 990.501 82.0699
    vertices.extend(cubic_bezier((889.763, 8.59805), (930.553, 21.2544), (965.987, 47.0971), (990.501, 82.0699)))
    # C1003.97 101.244 1013.73 122.773 1019.27 145.542
    vertices.extend(cubic_bezier((990.501, 82.0699), (1003.97, 101.244), (1013.73, 122.773), (1019.27, 145.542)))
    # C1020.95 152.68 1022.15 164.148 1024 170.155
    vertices.extend(cubic_bezier((1019.27, 145.542), (1020.95, 152.68), (1022.15, 164.148), (1024, 170.155)))
    
    # L1024 852.733
    vertices.append((1024, 852.733))
    
    # C1021.23 865.106 1020.16 877.182 1016.29 889.339
    vertices.extend(cubic_bezier((1024, 852.733), (1021.23, 865.106), (1020.16, 877.182), (1016.29, 889.339)))
    # C993.785 960.081 930.206 1014.24 856.048 1022.57
    vertices.extend(cubic_bezier((1016.29, 889.339), (993.785, 960.081), (930.206, 1014.24), (856.048, 1022.57)))
    # C853.815 1022.82 850.925 1023.34 848.781 1024
    vertices.extend(cubic_bezier((856.048, 1022.57), (853.815, 1022.82), (850.925, 1023.34), (848.781, 1024)))
    
    # L175.622 1024
    vertices.append((175.622, 1024))
    
    # C170.747 1022.44 156.998 1020.82 150.832 1019.49
    vertices.extend(cubic_bezier((175.622, 1024), (170.747, 1022.44), (156.998, 1020.82), (150.832, 1019.49)))
    # C129.157 1014.7 108.501 1006.1 89.8244 994.105
    vertices.extend(cubic_bezier((150.832, 1019.49), (129.157, 1014.7), (108.501, 1006.1), (89.8244, 994.105)))
    # C50.3102 968.655 21.4302 930.504 7.79391 885.391
    vertices.extend(cubic_bezier((89.8244, 994.105), (50.3102, 968.655), (21.4302, 930.504), (7.79391, 885.391)))
    # C3.65804 871.708 2.93576 859.785 0 846.258
    vertices.extend(cubic_bezier((7.79391, 885.391), (3.65804, 871.708), (2.93576, 859.785), (0, 846.258)))
    
    # L0 177.716
    vertices.append((0, 177.716))
    
    # C1.59147 172.71 3.35238 157.583 4.66099 151.222
    vertices.extend(cubic_bezier((0, 177.716), (1.59147, 172.71), (3.35238, 157.583), (4.66099, 151.222)))
    # C9.48764 128.179 18.6774 106.273 31.735 86.6826
    vertices.extend(cubic_bezier((4.66099, 151.222), (9.48764, 128.179), (18.6774, 106.273), (31.735, 86.6826)))
    # C58.0123 46.7943 98.6714 17.3211 144.837 5.72546
    vertices.extend(cubic_bezier((31.735, 86.6826), (58.0123, 46.7943), (98.6714, 17.3211), (144.837, 5.72546)))
    # C154.667 3.25625 164.36 2.0258 174.22 0
    vertices.extend(cubic_bezier((144.837, 5.72546), (154.667, 3.25625), (164.36, 2.0258), (174.22, 0)))

    # Apply translation and scale 0.85 to center squircle
    # Then multiply by scale_factor for the super-sampling high-resolution canvas
    SCALE = 0.85
    OFFSET = (1024 - 1024 * SCALE) / 2 # 76.8
    transformed_vertices = []
    for x, y in vertices:
        tx = (OFFSET + x * SCALE) * scale_factor
        ty = (OFFSET + y * SCALE) * scale_factor
        transformed_vertices.append((tx, ty))
        
    # Draw filled polygon on the large high-res canvas
    draw.polygon(transformed_vertices, fill=255)
    
    # Downscale with Lanczos for premium, ultra-smooth edges
    try:
        resample_filter = Image.Resampling.LANCZOS
    except AttributeError:
        resample_filter = Image.ANTIALIAS
        
    mask = mask.resize((size, size), resample=resample_filter)
    return mask, transformed_vertices

def get_gradient_color(y, size, is_dark):
    """
    Computes standard macOS-style 3D border gradient:
    Highlight at the top (light reflecting), transition in the middle, and soft shadow at the bottom.
    """
    SCALE = 0.85
    OFFSET = (1024 - 1024 * SCALE) / 2 # 76.8
    scale_factor = size / 1024.0
    y_start = OFFSET * scale_factor
    y_end = (OFFSET + 1024 * SCALE) * scale_factor
    if y <= y_start:
        t = 0.0
    elif y >= y_end:
        t = 1.0
    else:
        t = (y - y_start) / (y_end - y_start)
        
    if is_dark:
        # Dark mode gradient: Luminous white-silver highlight at the top, transition to transparent gray, to soft black shadow at bottom
        if t < 0.25:
            k = t / 0.25
            r, g, b = 255, 255, 255
            # Luminous rim lighting: higher top alpha for stronger edge separation in dark UI
            a = int(220 * (1 - k) + 80 * k)
        elif t < 0.75:
            k = (t - 0.25) / 0.50
            r = int(255 * (1 - k) + 0 * k)
            g = int(255 * (1 - k) + 0 * k)
            b = int(255 * (1 - k) + 0 * k)
            a = int(80 * (1 - k) + 40 * k) # slightly higher mid-rim opacity for better edge definition
        else:
            k = (t - 0.75) / 0.25
            r, g, b = 0, 0, 0
            a = int(40 * (1 - k) + 120 * k) # soft shadow at the bottom
    else:
        # Light mode gradient: Pure white highlight at the top, transitioning to subtle gray, to deep dark silver shadow at bottom
        if t < 0.25:
            k = t / 0.25
            r, g, b = 255, 255, 255
            a = int(220 * (1 - k) + 100 * k)
        elif t < 0.75:
            k = (t - 0.25) / 0.50
            r = int(255 * (1 - k) + 140 * k)
            g = int(255 * (1 - k) + 140 * k)
            b = int(255 * (1 - k) + 140 * k)
            a = 100 # slightly higher mid-rim opacity for better edge definition
        else:
            k = (t - 0.75) / 0.25
            r = int(140 * (1 - k) + 90 * k) # deeper dark silver shadow at bottom to avoid washed-out edges
            g = int(140 * (1 - k) + 90 * k)
            b = int(140 * (1 - k) + 90 * k)
            a = int(100 * (1 - k) + 220 * k) # higher bottom opacity for deep shadows
            
    return (r, g, b, a)

def generate_bezel_overlay(size, transformed_vertices, is_dark, scale_factor=4):
    """
    Renders a super-sampled vertical-gradient bezel/border overlay that runs exactly
    on the squircle contour to simulate the 3D Apple App Icon border design.
    """
    large_size = size * scale_factor
    
    # 1. Create a 1D gradient strip and scale it to full width
    gradient_strip = Image.new("RGBA", (1, large_size))
    for y in range(large_size):
        gradient_strip.putpixel((0, y), get_gradient_color(y, large_size, is_dark))
    gradient = gradient_strip.resize((large_size, large_size))
    
    # 2. Draw outline on stroke mask
    stroke_mask = Image.new("L", (large_size, large_size), 0)
    draw_stroke = ImageDraw.Draw(stroke_mask)
    
    # At 1024x1024, a 1.5px border is extremely crisp and elegant.
    # At 4x scale, this is 6.0px.
    stroke_width = 6
    draw_stroke.polygon(transformed_vertices, outline=255, width=stroke_width)
    
    # 3. Composite gradient using the outline mask
    bezel_overlay = Image.new("RGBA", (large_size, large_size), (0, 0, 0, 0))
    bezel_overlay = Image.composite(gradient, bezel_overlay, stroke_mask)
    
    # 4. Downsample with Lanczos to get mathematically perfect anti-aliasing
    try:
        resample_filter = Image.Resampling.LANCZOS
    except AttributeError:
        resample_filter = Image.ANTIALIAS
        
    bezel_overlay_1024 = bezel_overlay.resize((size, size), resample=resample_filter)
    return bezel_overlay_1024

def make_transparent(img_path, out_path, is_dark):
    img = Image.open(img_path).convert("RGBA")
    width, height = img.size
    
    # 1. Generate mathematically perfect squircle mask and high-res path vertices
    mask, vertices = generate_squircle_mask(size=width)
    
    # 2. Apply initial squircle mask to the raw image (removes original block background)
    r, g, b, a = img.split()
    new_a = ImageChops.darker(a, mask)
    masked_img = Image.merge("RGBA", (r, g, b, new_a))
    
    # 3. Generate the 3D Apple-style gradient bezel stroke
    bezel = generate_bezel_overlay(width, vertices, is_dark)
    
    # 4. Overlay the bezel stroke exactly on top of the squircle
    composited_img = Image.alpha_composite(masked_img, bezel)
    
    # 5. Re-apply the squircle mask to trim the outer 50% of the bezel stroke
    # This guarantees the background outside is 100% transparent and the border remains perfectly crisp inside.
    cr, cg, cb, ca = composited_img.split()
    final_a = ImageChops.darker(ca, mask)
    final_img = Image.merge("RGBA", (cr, cg, cb, final_a))
    
    # Save the flawless, premium icon
    final_img.save(out_path, "PNG")
    print(f"Flawless 3D-beveled transparent squircle saved to {out_path} successfully!")

if __name__ == "__main__":
    # We load from the original raw enlarged templates in the artifacts directory to avoid accumulating alpha degradation
    raw_dark = "/Users/stormix/.gemini/antigravity/brain/31f6911d-68df-433a-b949-05d480acecaf/ferrumgrid_icon_dark_enlarged_1779454847190.png"
    raw_light = "/Users/stormix/.gemini/antigravity/brain/31f6911d-68df-433a-b949-05d480acecaf/ferrumgrid_icon_light_grid_enlarged_1779454822306.png"
    
    # Process Dark Mode Icon
    if os.path.exists(raw_dark):
        make_transparent(raw_dark, "assets/app-icon.png", is_dark=True)
    else:
        # Fallback if raw template isn't found
        make_transparent("assets/app-icon.png", "assets/app-icon.png", is_dark=True)
        
    # Process Light Mode Icon
    if os.path.exists(raw_light):
        make_transparent(raw_light, "assets/app-icon-light.png", is_dark=False)
    else:
        # Fallback
        if os.path.exists("assets/app-icon-light.png"):
            make_transparent("assets/app-icon-light.png", "assets/app-icon-light.png", is_dark=False)
