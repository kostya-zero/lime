# 🍋‍🟩 Lime

Lime is a lightweight HTTP server that hosts HTML and assets for websites. It is designed to be easy to deploy and use, making it ideal for static sites or simple web applications.

## Installation

The only way to install Lime is to download a pre-built binary from the [releases page](https://github.com/kostya-zero/lime/releases). The binary is available for various Linux and Windows.

## Usage

Lime can serve your HTML app with `serve` subcommand:

```bash
lime serve
```

**But**, It will lookup for two important directories:

- `static` - for static assets like CSS, JS, images, etc.
- `pages` - for HTML files.

So you need to separate your assets from your HTML files and put them into these directories.

## Configuration

You can configure this behavior by creating a `lime.toml` file in the root of your project. Here is an example configuration:

```toml
host= "127.0.01"
port = 3000
pages_dir = "./pages"
static_dir = "./static"
```

## Contributing

Make a pull request...

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
```

