import os
from PIL import Image, ImageDraw, ImageFont

def create_vertical_gradient(w, h, color1, color2):
    base = Image.new('RGBA', (w, h))
    draw = ImageDraw.Draw(base)
    for y in range(h):
        t = y / (h - 1)
        r = int(color1[0] * (1 - t) + color2[0] * t)
        g = int(color1[1] * (1 - t) + color2[1] * t)
        b = int(color1[2] * (1 - t) + color2[2] * t)
        a = 255
        draw.line([(0, y), (w, y)], fill=(r, g, b, a))
    return base

def main():
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    assets_dir = os.path.join(root_dir, "assets")
    os.makedirs(assets_dir, exist_ok=True)
    
    # Standard crisp macOS DMG dimensions (Retina scale 1280x800 for 640x400 window)
    W, H = 1280, 800
    
    print("Generating premium DMG background canvas...")
    # 1. Dark Theme Gradient Background
    bg = create_vertical_gradient(W, H, (28, 29, 32), (16, 17, 20))
    draw = ImageDraw.Draw(bg)
    
    # 2. Draw Subtle Brand Grid Pattern in background (very elegant)
    grid_spacing = 80
    grid_color = (255, 255, 255, 4)  # Barely visible white grid lines
    for x in range(0, W, grid_spacing):
        draw.line([(x, 0), (x, H)], fill=grid_color, width=1)
    for y in range(0, H, grid_spacing):
        draw.line([(0, y), (W, y)], fill=grid_color, width=1)
        
    # 3. Placement slot guides for Icons (Left: App, Right: Applications folder)
    # Target positions in 640x400 coords: Left x=160, Right x=480, y=220
    # In 1280x800 coords: Left x=320, Right x=960, y=440
    slot_size = 180
    
    left_slot_center = (320, 440)
    right_slot_center = (960, 440)
    
    # Left guides (subtle neon-mint dashed border slot for the App)
    lx0, ly0 = left_slot_center[0] - slot_size//2, left_slot_center[1] - slot_size//2
    lx1, ly1 = left_slot_center[0] + slot_size//2, left_slot_center[1] + slot_size//2
    draw.rounded_rectangle([lx0, ly0, lx1, ly1], radius=32, outline=(62, 207, 142, 24), width=2, fill=(255, 255, 255, 2))
    
    # Right guides (subtle white/silver dashed border slot for Applications)
    rx0, ry0 = right_slot_center[0] - slot_size//2, right_slot_center[1] - slot_size//2
    rx1, ry1 = right_slot_center[0] + slot_size//2, right_slot_center[1] + slot_size//2
    draw.rounded_rectangle([rx0, ry0, rx1, ry1], radius=32, outline=(255, 255, 255, 12), width=2, fill=(255, 255, 255, 2))
    
    # 4. Premium Neon Drag-and-Drop Arrow
    # Arrow spans from x=460 to x=820, y=440
    arrow_y = 440
    arrow_start_x = 460
    arrow_end_x = 820
    
    # Subtle dashed guide line
    draw.line([(arrow_start_x, arrow_y), (arrow_end_x, arrow_y)], fill=(62, 207, 142, 60), width=4)
    # Beautiful modern glow arrowhead
    arrowhead_len = 24
    draw.polygon([
        (arrow_end_x, arrow_y),
        (arrow_end_x - arrowhead_len, arrow_y - arrowhead_len * 0.7),
        (arrow_end_x - arrowhead_len * 0.5, arrow_y),
        (arrow_end_x - arrowhead_len, arrow_y + arrowhead_len * 0.7)
    ], fill=(62, 207, 142, 180))
    
    # 5. Premium Typography (using high-quality PIL default fallback or system fonts if available)
    title_text = "FerrumGrid"
    subtitle_text = "Drag FerrumGrid to the Applications folder to install"
    
    # Let's try loading a system font (San Francisco or Arial)
    font_path_bold = "/System/Library/Fonts/HelveticaNeue.ttc"
    font_path_regular = "/System/Library/Fonts/HelveticaNeue.ttc"
    
    title_font = None
    sub_font = None
    
    if os.path.exists(font_path_bold):
        try:
            # Under macOS, index 1 or 2 is bold/regular
            title_font = ImageFont.truetype(font_path_bold, 64, index=1)
            sub_font = ImageFont.truetype(font_path_regular, 32, index=0)
        except Exception:
            pass
            
    if title_font is None:
        title_font = ImageFont.load_default()
        sub_font = ImageFont.load_default()
        print("Warning: Falling back to default PIL font.")
        
    # Draw Text centered
    if hasattr(draw, "textbbox"):
        # Modern pillow text centering
        t_box = draw.textbbox((0, 0), title_text, font=title_font)
        t_w = t_box[2] - t_box[0]
        s_box = draw.textbbox((0, 0), subtitle_text, font=sub_font)
        s_w = s_box[2] - s_box[0]
    else:
        # Fallback for old pillow
        t_w, _ = draw.textsize(title_text, font=title_font)
        s_w, _ = draw.textsize(subtitle_text, font=sub_font)
        
    draw.text((W // 2 - t_w // 2, 140), title_text, font=title_font, fill=(255, 255, 255, 230))
    draw.text((W // 2 - s_w // 2, 225), subtitle_text, font=sub_font, fill=(150, 150, 155, 180))
    
    # Save image
    out_path = os.path.join(assets_dir, "dmg-background.png")
    # Resize with premium Lanczos to generate Retina 2x scale and regular 1x scale if needed,
    # but macOS Finder natively handles high-res background perfectly when set!
    bg.save(out_path, "PNG")
    print(f"Flawless premium DMG background saved to {out_path} successfully!")

if __name__ == "__main__":
    main()
