# 🍋‍🟩 Lime

Lime is a simple, lightweight, and fast HTTP server designed to host static websites and simple web applications. It's built for ease of use and quick deployment, making it an ideal choice for developers looking for a no-fuss solution for serving HTML and static assets.

## Features

- **Zero-Configuration by Default**: Runs out of the box without needing a configuration file.
- **Simple & Fast**: Built with performance in mind to serve static content efficiently.
- **Easy Deployment**: Single binary installation for Linux and Windows.
- **Customizable**: Configure the host, port, and directory paths via a simple lime.toml file.

## Getting started

Follow these steps to get your Lime server up and running.

### Installation

The recommended way to install Lime is to download a pre-built binary from the official [releases page](https://github.com/kostya-zero/lime/releases) and put it in the directory that exists in `PATH`. 
Binaries are available for Linux and Windows.

### Project Structure

Before running the server, it's important to organize your website's files into two separate directories:

 1. **Pages Directory**: Contains your HTML files (e.g., `index.html`, `about.html`).
 2. **Static Assets Directory**: Contains your CSS, JavaScript, images, and other static files (e.g., `style.css`, `app.js`).

By default, Lime looks for a `pages` directory for HTML and a `static` directory for assets.

This separation is crucial because of Lime's routing logic.
When a request comes in, Lime inspects the file extension.
If the request path has no extension or ends with `.html`, Lime searches for the corresponding file in the `pages` directory.
For all other file extensions (like `.css`, `.js`, `.png`, etc.), it searches in the `static` directory.
This allows you to use clean URLs in your HTML, like `/css/style.css`, and Lime will correctly resolve the path to `./static/css/style.css`.

## Configuration

While Lime works without any configuration, you can customize its behavior by creating a `lime.toml` file in the root of your project.
Here is an example configuration:

```toml
host= "127.0.0.1"
port = 3000
pages_dir = "./pages"
static_dir = "./static"
```

## Contributing

Make a pull request...

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
