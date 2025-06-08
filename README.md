# AQIV - arabian's Quick Image Viewer

A fast, lightweight, and feature-rich image viewer built with Rust and egui.

## Features

- **Fast Performance**: Built with Rust for maximum speed and efficiency
- **Wide Format Support**: Supports AVIF, BMP, DDS, GIF, HDR, ICO, JPEG, EXR, PNG, PNM, QOI, TGA, TIFF, WebP, and SVG
- **Intuitive Controls**: Easy-to-use keyboard shortcuts for all operations
- **Image Manipulation**:
    - Zoom in/out with mouse wheel or keyboard
    - Pan images by dragging
    - Rotate images in 90° increments
    - Flip images horizontally and vertically
- **Smart Window Sizing**: Automatically adjusts window size based on image dimensions and screen size
- **Image Information Display**: View detailed file information including format, size, resolution, and path
- **Customizable**: Clean, dark interface with notification system
- **Cross-platform**: Works on Windows, macOS, and Linux

## Installation (using cargo)

```bash
# Install globally
cargo install aqiv

# Or install directly from Git
cargo install --git https://github.com/arabianq/aqiv.git
```

## Build from Source

#### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- Git

#### Building

```bash
# Clone the repository
git clone https://github.com/arabianq/aqiv.git
cd aqiv

# Build in release mode for optimal performance
cargo build --release

# The binary will be available at target/release/aqiv (or aqiv.exe on Windows)
```

## Usage

### Basic Usage

```bash
# View an image
aqiv path/to/your/image.jpg

# Examples
aqiv photo.png
aqiv ~/Pictures/vacation.jpeg
aqiv "C:\Users\Name\Desktop\image with spaces.gif"
```

### Keyboard Controls

| Key           | Action                                 |
|---------------|----------------------------------------|
| `Escape`      | Exit the application                   |
| `Mouse Wheel` | Zoom in/out (centered on mouse cursor) |
| `W`           | Zoom in                                |
| `S`           | Zoom out                               |
| `Mouse Drag`  | Pan/move the image                     |
| `R`           | Rotate image 90° clockwise             |
| `H`           | Flip image horizontally                |
| `V`           | Flip image vertically                  |
| `C`           | Reset image position to center         |
| `X`           | Reset zoom to 100%                     |
| `D`           | Toggle maintain aspect ratio           |
| `I`           | Toggle image information display       |

## Configuration

AQIV uses sensible defaults, but you can modify the source code to customize:

- Background color (default: dark gray `#1B1B1B`)
- Default aspect ratio maintenance (default: enabled)
- Notification duration (default: 500ms)
- Initial info display state (default: hidden)
