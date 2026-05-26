import sys
import os
from PIL import Image, ImageDraw, ImageFilter

# Add scripts directory to path to import make_transparent
sys.path.append(os.path.dirname(__file__))
from make_transparent import make_transparent

def create_vertical_gradient(size, color1, color2):
    """
    Creates a simple top-to-bottom vertical gradient for the squircle background.
    """
    base = Image.new('RGBA', (size, size))
    draw = ImageDraw.Draw(base)
    for y in range(size):
        t = y / (size - 1)
        r = int(color1[0] * (1 - t) + color2[0] * t)
        g = int(color1[1] * (1 - t) + color2[1] * t)
        b = int(color1[2] * (1 - t) + color2[2] * t)
        a = 255
        draw.line([(0, y), (size, y)], fill=(r, g, b, a))
    return base

def create_diagonal_gradient(size, color1, color2, start_x, start_y, grid_w):
    """
    Creates a mathematically continuous diagonal gradient from top-left to bottom-right
    of the grid bounding box.
    """
    base = Image.new('RGBA', (size, size))
    draw = ImageDraw.Draw(base)
    
    min_c = start_x + start_y
    max_c = start_x + start_y + 2 * grid_w
    
    for c_val in range(size * 2):
        if c_val <= min_c:
            t = 0.0
        elif c_val >= max_c:
            t = 1.0
        else:
            t = (c_val - min_c) / (max_c - min_c)
        
        r = int(color1[0] * (1 - t) + color2[0] * t)
        g = int(color1[1] * (1 - t) + color2[1] * t)
        b = int(color1[2] * (1 - t) + color2[2] * t)
        
        # Draw a perpendicular line at x + y = c_val
        x0 = max(0, c_val - (size - 1))
        y0 = c_val - x0
        x1 = min(size - 1, c_val)
        y1 = c_val - x1
        draw.line([(x0, y0), (x1, y1)], fill=(r, g, b, 255), width=2)
        
    return base

def draw_grid_f_continuous_gradient(image, color1, color2, size=125, spacing=25, radius=20, is_dark=False):
    """
    Draws the "F" shape consisting of 9 rounded rectangle blocks, filled with a
    seamless, continuous diagonal gradient across the entire F, complete with a
    soft, realistic 3D drop shadow.
    """
    # 1. Create a 1-bit/grayscale mask for the F blocks
    mask = Image.new('L', (1024, 1024), 0)
    draw_mask = ImageDraw.Draw(mask)
    
    start_x = (1024 - (4 * size + 3 * spacing)) // 2  # 224
    start_y = (1024 - (4 * size + 3 * spacing)) // 2  # 224
    grid_w = 4 * size + 3 * spacing                   # 575
    
    # Row 0: 4 blocks, Row 1: 1 block, Row 2: 3 blocks, Row 3: 1 block
    blocks = [
        (0, 0), (1, 0), (2, 0), (3, 0),
        (0, 1),
        (0, 2), (1, 2), (2, 2),
        (0, 3)
    ]
    
    for c, r in blocks:
        x = start_x + c * (size + spacing)
        y = start_y + r * (size + spacing)
        draw_mask.rounded_rectangle(
            [x, y, x + size, y + size],
            radius=radius,
            fill=255
        )
        
    # 2. Render realistic 3D Drop Shadow
    # Light Mode: deeper and tighter shadow for bright wallpapers
    # Dark Mode: softer, wider shadow for organic low-contrast absorption
    shadow_opacity = 0.22 if is_dark else 0.35
    shadow_blur = 20 if is_dark else 15
    shadow_offset_y = 6 if is_dark else 8
    
    # Create black shadow alpha layer
    shadow_img = Image.new('RGBA', (1024, 1024), (0, 0, 0, 0))
    black_fill = Image.new('RGBA', (1024, 1024), (0, 0, 0, int(255 * shadow_opacity)))
    shadow_img.paste(black_fill, (0, 0), mask=mask)
    
    # Apply Gaussian Blur filter for flawless drop shadow softness
    blurred_shadow = shadow_img.filter(ImageFilter.GaussianBlur(shadow_blur))
    
    # Offset the shadow vertically downwards to simulate direct top lighting
    offset_shadow = Image.new('RGBA', (1024, 1024), (0, 0, 0, 0))
    offset_shadow.paste(blurred_shadow, (0, shadow_offset_y))
    
    # Draw shadow on the parent image before drawing the F shapes
    image.alpha_composite(offset_shadow)
        
    # 3. Create the continuous diagonal gradient image aligned to the F bounding box
    gradient_img = create_diagonal_gradient(1024, color1, color2, start_x, start_y, grid_w)
    
    # 4. Paste the continuous gradient onto the image using the mask
    image.paste(gradient_img, (0, 0), mask=mask)

def main():
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    assets_dir = os.path.join(root_dir, "assets")
    os.makedirs(assets_dir, exist_ok=True)
    
    print("Generating raw dark mode canvas with continuous F gradient...")
    # Process Dark Mode Icon
    dark_raw = Image.new('RGBA', (1024, 1024), (0,0,0,0))
    dark_bg = create_vertical_gradient(1024, (31, 32, 33), (15, 16, 17))
    dark_raw = Image.alpha_composite(dark_raw, dark_bg)
    # Continuous subtle white-to-silver diagonal gradient (matches light mode F size exactly)
    draw_grid_f_continuous_gradient(dark_raw, (255, 255, 255), (210, 210, 215), size=135, spacing=25, radius=20, is_dark=True)
    dark_temp_path = os.path.join(root_dir, "dark_temp_raw.png")
    dark_raw.save(dark_temp_path)
    
    dark_out_path = os.path.join(assets_dir, "app-icon.png")
    make_transparent(dark_temp_path, dark_out_path, is_dark=True)
    if os.path.exists(dark_temp_path):
        os.remove(dark_temp_path)
        
    print("Generating raw light mode canvas with continuous F gradient...")
    light_raw = Image.new('RGBA', (1024, 1024), (0,0,0,0))
    # White-to-light-gray gradient per Apple HIG (#FFFFFF pure white at top to #E5E5EA soft light gray at bottom)
    light_bg = create_vertical_gradient(1024, (255, 255, 255), (229, 229, 234))
    light_raw = Image.alpha_composite(light_raw, light_bg)
    # Continuous subtle slate to charcoal diagonal gradient (scaled slightly larger for premium contrast)
    draw_grid_f_continuous_gradient(light_raw, (58, 58, 66), (26, 26, 32), size=135, spacing=25, radius=20)
    light_temp_path = os.path.join(root_dir, "light_temp_raw.png")
    light_raw.save(light_temp_path)
    
    light_out_path = os.path.join(assets_dir, "app-icon-light.png")
    make_transparent(light_temp_path, light_out_path, is_dark=False)
    if os.path.exists(light_temp_path):
        os.remove(light_temp_path)
        
    print("Successfully completed icon generation!")

if __name__ == "__main__":
    main()
