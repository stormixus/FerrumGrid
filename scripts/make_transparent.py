import os
import sys
from PIL import Image, ImageChops

def make_transparent(img_path, out_path):
    img = Image.open(img_path).convert("RGBA")
    width, height = img.size
    pixels = img.load()
    
    # BFS to find background pixels
    visited = set()
    queue = []
    
    # Seed queue with the borders (outer 5 pixels)
    for x in range(width):
        for y in range(5):
            queue.append((x, y))
            visited.add((x, y))
            queue.append((x, height - 1 - y))
            visited.add((x, height - 1 - y))
    for y in range(height):
        for x in range(5):
            if (x, y) not in visited:
                queue.append((x, y))
                visited.add((x, y))
            if (width - 1 - x, y) not in visited:
                queue.append((width - 1 - x, y))
                visited.add((width - 1 - x, y))
                
    threshold = 28 # Brightness threshold for black background
    
    background_pixels = set(queue)
    head = 0
    while head < len(queue):
        x, y = queue[head]
        head += 1
        
        for dx, dy in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
            nx, ny = x + dx, y + dy
            if 0 <= nx < width and 0 <= ny < height and (nx, ny) not in visited:
                r, g, b, a = pixels[nx, ny]
                # If it is dark enough, it belongs to the background
                # We use a threshold of 28 for RGB values
                if r < threshold and g < threshold and b < threshold:
                    visited.add((nx, ny))
                    background_pixels.add((nx, ny))
                    queue.append((nx, ny))
                    
    # Now we modify pixels
    # For background pixels, we make them fully transparent.
    # To avoid jagged edges, we can do a slight anti-aliasing:
    # Any pixel on the boundary of the background and the foreground gets a semi-transparent alpha.
    for x, y in background_pixels:
        r, g, b, a = pixels[x, y]
        pixels[x, y] = (0, 0, 0, 0)
        
    # Apply a gentle anti-aliasing on the edges
    # We find all non-background pixels that have at least one background neighbor,
    # and adjust their alpha to smooth the transition.
    for x in range(1, width - 1):
        for y in range(1, height - 1):
            if (x, y) not in background_pixels:
                # Count background neighbors
                bg_neighbors = 0
                for dx, dy in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)]:
                    if (x + dx, y + dy) in background_pixels:
                        bg_neighbors += 1
                if bg_neighbors > 0:
                    r, g, b, a = pixels[x, y]
                    # Reduce alpha slightly based on background neighbors to anti-alias
                    # If it has 8 bg neighbors, it is almost background (low alpha)
                    # If it has 1, it is almost foreground (high alpha)
                    new_a = int(a * (1.0 - 0.1 * bg_neighbors))
                    pixels[x, y] = (r, g, b, max(0, new_a))
                    
    # Save the result
    img.save(out_path, "PNG")
    print(f"Transparency applied and saved to {out_path} successfully!")

if __name__ == "__main__":
    make_transparent("assets/app-icon.png", "assets/app-icon.png")
    if os.path.exists("assets/app-icon-light.png"):
        make_transparent("assets/app-icon-light.png", "assets/app-icon-light.png")
