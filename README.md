# JSON Echo

A powerful and flexible mock API server for rapid prototyping, testing, and development. JSON Echo allows you to create realistic API responses from simple JSON configuration files, making it perfect for frontend development, API testing, and service mocking.

## 🚀 Features

- **JSON-Based Configuration**: Define routes and responses using intuitive JSON files
- **Dynamic Route Handling**: Support for parameterized routes with flexible data querying
- **Static File Serving**: Built-in static file serving for assets, images, and frontend files
- **CORS Support**: Built-in cross-origin resource sharing for web applications
- **Async Performance**: High-performance async server built with Tokio and Axum
- **Type-Safe**: Written in Rust with comprehensive error handling
- **Auto-Discovery**: Automatic project root detection and configuration file discovery
- **Extensible**: Modular architecture for easy customization and extension

## 📦 Installation

### From Source

```bash
git clone https://github.com/your-org/json-echo.git
cd json-echo
cargo build --release
```

The binary will be available at `target/release/echo`.

### Using Cargo

```bash
cargo install --path .
```

## 🏃‍♂️ Quick Start

### 1. Initialize a New Project

```bash
echo init
```

This creates a `json-echo.json` configuration file with default settings:

```json
{
  "port": 3001,
  "hostname": "localhost",
  "static_folder": "public",
  "static_route": "/static",
  "routes": {}
}
```

### 2. Configure Your API Routes

Edit the `json-echo.json` file to define your mock API:

```json
{
  "port": 3001,
  "hostname": "localhost",
  "static_folder": "public",
  "static_route": "/static",
  "routes": {
    "/api/users": {
      "method": "GET",
      "description": "Get all users",
      "id_field": "id",
      "response": {
        "status": 200,
        "body": [
          {"id": 1, "name": "John Doe", "email": "john@example.com"},
          {"id": 2, "name": "Jane Smith", "email": "jane@example.com"}
        ]
      }
    },
    "[GET] /api/users/{id}": {
      "description": "Get user by ID",
      "id_field": "id",
      "response": "users.json"
    }
  }
}
```

### 3. Start the Server

```bash
echo serve
```

Your mock API is now running at `http://localhost:3001`!

## 📖 Usage

### Command Line Interface

#### Global Options

- `--config <PATH>`: Path to configuration file (default: `json-echo.json`)
- `--log-level <LEVEL>`: Set logging level (`trace`, `debug`, `info`, `warn`, `error`)
- `--protocol <PROTOCOL>`: Network protocol (default: `http`)

#### Commands

##### `init`
Initialize a new JSON Echo project with default configuration.

```bash
echo init
```

**Options:**
- Creates `json-echo.json` in the current directory
- Sets up basic server configuration
- Ready for immediate customization

##### `serve`
Start the mock API server with the specified configuration.

```bash
echo serve
```

**Behavior:**
- Loads configuration from the specified file
- Starts HTTP server on configured host and port
- Serves mock responses based on route definitions
- Supports hot-reloading during development

### Configuration Examples

#### Basic API with Multiple Routes

```json
{
  "port": 8080,
  "hostname": "0.0.0.0",
  "static_folder": "assets",
  "static_route": "/public",
  "routes": {
    "/api/products": {
      "method": "GET",
      "description": "Product catalog",
      "response": {
        "status": 200,
        "body": {
          "products": [
            {"id": 1, "name": "Laptop", "price": 999.99},
            {"id": 2, "name": "Mouse", "price": 29.99}
          ],
          "total": 2
        }
      }
    },
    "/api/health": {
      "method": "GET",
      "response": {
        "status": 200,
        "body": {"status": "ok", "timestamp": "2024-01-01T00:00:00Z"}
      }
    }
  }
}
```

#### External File References

```json
{
  "routes": {
    "/api/users": {
      "method": "GET",
      "response": "data/users.json"
    },
    "/api/posts": {
      "method": "GET",
      "response": "data/posts.json"
    }
  }
}
```

#### Parameterized Routes

```json
{
  "static_folder": "www",
  "static_route": "/assets",
  "routes": {
    "/api/users/{id}": {
      "method": "GET",
      "id_field": "user_id",
      "results_field": "data",
      "response": {
        "status": 200,
        "body": {
          "data": [
            {"user_id": 1, "name": "Alice"},
            {"user_id": 2, "name": "Bob"}
          ]
        }
      }
    }
  }
}
```

#### Static File Serving

```json
{
  "port": 3001,
  "hostname": "localhost",
  "static_folder": "public",
  "static_route": "/static",
  "routes": {
    "/api/data": {
      "method": "GET",
      "response": {
        "status": 200,
        "body": {"message": "API endpoint"}
      }
    }
  }
}
```

#### With HTTP Method

```json
{
  "routes": {
    "[GET] /api/users": {
      "response": "data/users.json"
    },
    "[POST] /api/users": {
      "response": "data/users.json"
    },
    "[DELETE] /api/users/{id}": {
      "response": "data/users.json"
    },
  }
}
```

With this configuration:
- Static files in the `public/` folder are served at `/static/*`
- `public/index.html` becomes available at `http://localhost:3001/static/index.html`
- `public/css/style.css` becomes available at `http://localhost:3001/static/css/style.css`
- API routes continue to work normally

### API Examples

Once your server is running, you can make requests:

```bash
# Get all users
curl http://localhost:3001/api/users

# Get specific user by ID
curl http://localhost:3001/api/users/1

# Health check
curl http://localhost:3001/api/health

# Access static files
curl http://localhost:3001/static/index.html
curl http://localhost:3001/static/css/style.css
curl http://localhost:3001/static/images/logo.png
```

## 📂 Static File Serving

JSON Echo includes built-in static file serving capabilities, allowing you to serve assets, frontend applications, and other static content alongside your mock API endpoints.

### Configuration

Enable static file serving by adding two optional fields to your configuration:

```json
{
  "port": 3001,
  "hostname": "localhost",
  "static_folder": "public",
  "static_route": "/static",
  "routes": {
    "/api/users": {
      "method": "GET",
      "response": {"status": 200, "body": []}
    }
  }
}
```

### Directory Structure

With the above configuration, organize your files like this:

```
your-project/
├── json-echo.json          # Configuration file
├── public/                 # Static files folder
│   ├── index.html         # Available at /static/index.html
│   ├── css/
│   │   └── style.css      # Available at /static/css/style.css
│   ├── js/
│   │   └── app.js         # Available at /static/js/app.js
│   └── images/
│       └── logo.png       # Available at /static/images/logo.png
└── data/
    └── users.json         # For API responses
```

### Use Cases

#### Development Environment

Mix static assets with API mocking during development:

```json
{
  "static_folder": "assets",
  "static_route": "/assets",
  "routes": {
    "/api/config": {
      "response": {"apiUrl": "http://localhost:3001", "environment": "development"}
    }
  }
}
```

#### Documentation Site

Serve API documentation alongside mock endpoints:

```json
{
  "static_folder": "docs",
  "static_route": "/docs",
  "routes": {
    "/api/openapi": {
      "response": "openapi.json"
    }
  }
}
```

### Advanced Configuration

#### Custom Route Paths

```json
{
  "static_folder": "web",
  "static_route": "/app",
  "routes": {}
}
```

Files in `web/` folder become available at `/app/*` URLs.

#### Root Path Serving

```json
{
  "static_folder": "build",
  "static_route": "/",
  "routes": {
    "/api/*": "..."
  }
}
```

Serves files at the root path while API routes are still available.

### File Type Support

Static file serving supports all common file types:

- **Web Assets**: `.html`, `.css`, `.js`, `.json`
- **Images**: `.png`, `.jpg`, `.jpeg`, `.gif`, `.svg`, `.ico`
- **Fonts**: `.woff`, `.woff2`, `.ttf`, `.otf`
- **Documents**: `.pdf`, `.txt`, `.md`
- **Media**: `.mp4`, `.webm`, `.mp3`, `.wav`

Content-Type headers are automatically set based on file extensions.

## 🏗️ Architecture

JSON Echo is built with a modular architecture:

```
json-echo/
├── crates/
│   ├── cli/           # Command-line interface
│   └── core/          # Core library functionality
├── examples/          # Configuration examples
│   ├── basic-api.json # Simple API example
│   ├── external-files.json # External file references
│   └── data/          # Sample data files
├── json-echo.json     # Default configuration
└── schema.json        # JSON schema for validation
```

### Core Library

The `json-echo-core` crate provides the foundational components:

- **Configuration Management**: JSON parsing and validation
- **Database System**: In-memory data storage and querying
- **Filesystem Operations**: Cross-platform file handling
- **Error Handling**: Comprehensive error types and propagation

For detailed API documentation and usage examples, see the [Core Library README](./crates/core/README.md).

### CLI Application

The `json-echo-cli` crate provides the user-facing command-line interface:

- **Argument Parsing**: Command-line option handling
- **Server Management**: HTTP server lifecycle
- **Route Handling**: Dynamic route generation and request processing
- **Middleware**: CORS support and error handling

## 🔧 Configuration Reference

### Server Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `port` | number | `3001` | Port number for the HTTP server |
| `hostname` | string | `"localhost"` | Hostname or IP address to bind to |
| `static_folder` | string | `null` | Relative folder path to serve static files from |
| `static_route` | string | `"/static"` | Base route path for serving static files |

### Route Configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `method` | string | No | HTTP method (default: `"GET"`) |
| `description` | string | No | Human-readable route description |
| `headers` | object | No | Custom HTTP headers to include |
| `id_field` | string | No | Field name for unique identifiers (default: `"id"`) |
| `results_field` | string | No | Field containing results when data is nested |
| `response` | object/string | Yes | Response configuration or file path |

### Response Configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `status` | number | No | HTTP status code (default: `200`) |
| `body` | any | No | Response body content |

## 🚀 Advanced Usage

### Multiple Configuration Files

```bash
# Use example configurations
echo --config examples/basic-api.json serve

# External file example
echo --config examples/external-files.json serve

# Custom configuration
echo --config my-custom-config.json serve
```

### Static File Serving Examples

```bash
# Development with assets
echo --config dev-config.json serve

# Documentation site
echo --config docs-config.json serve
```

Example configurations:

**spa-config.json** - Serve SPA at root:
```json
{
  "port": 3000,
  "static_folder": "dist",
  "static_route": "/",
  "routes": {
    "/api/auth": {"response": {"token": "mock-token"}},
    "[GET] /api/users": {"response": "data/users.json"}
  }
}
```

**dev-config.json** - Development setup:
```json
{
  "static_folder": "public",
  "static_route": "/assets",
  "routes": {
    "/api/config": {"response": {"env": "development"}},
    "/api/data/{id}": {"response": "data/items.json"}
  }
}
```

### Custom Logging

```bash
# Debug mode for development
echo --log-level debug serve

# Minimal logging for production
echo --log-level warn serve
```

### Docker Support

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/echo /usr/local/bin/
COPY config.json /app/
COPY public/ /app/public/
COPY data/ /app/data/
WORKDIR /app
EXPOSE 3001
CMD ["echo", "serve"]
```

## 🧪 Testing

### Unit Tests

```bash
cargo test
```

### Individual Tests

```bash
cargo test -p json-echo-core --test config_tests
cargo test -p json-echo-core --test filesystem_tests
cargo test -p json-echo-core --test database_tests
```

### Example Requests

Test your configuration with example requests:

```bash
# Test basic endpoint
curl -X GET http://localhost:3001/api/users

# Test parameterized endpoint
curl -X GET http://localhost:3001/api/users/1

# Test health check
curl -X GET http://localhost:3001/api/health

# Test static file serving
curl -X GET http://localhost:3001/static/index.html
curl -X GET http://localhost:3001/static/css/style.css
curl -I http://localhost:3001/static/images/logo.png

# Test CORS headers
curl -X OPTIONS http://localhost:3001/api/users \
  -H "Origin: http://localhost:3000" \
  -H "Access-Control-Request-Method: GET"

# Test static files with CORS
curl -X OPTIONS http://localhost:3001/static/app.js \
  -H "Origin: http://localhost:3000" \
  -H "Access-Control-Request-Method: GET"
```

### Static File Testing

Create a test directory structure:

```bash
mkdir -p public/{css,js,images}
echo "<h1>Hello World</h1>" > public/index.html
echo "body { margin: 0; }" > public/css/style.css
echo "console.log('Hello');" > public/js/app.js
```

Test configuration:
```json
{
  "static_folder": "public",
  "static_route": "/static",
  "routes": {
    "/api/test": {"response": {"message": "API works"}}
  }
}
```

Verify both API and static content:
```bash
# Test API endpoint
curl http://localhost:3001/api/test

# Test static files
curl http://localhost:3001/static/index.html
curl http://localhost:3001/static/css/style.css
curl http://localhost:3001/static/js/app.js
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

1. Clone the repository
2. Install Rust (1.70.0 or later)
3. Run tests: `cargo test`
4. Run the CLI: `cargo run --bin echo -- --help`

### Code Style

- Follow Rust standard formatting: `cargo fmt`
- Ensure clippy passes: `cargo clippy`
- Add tests for new functionality
- Update documentation for API changes

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- [Core Library Documentation](./crates/core/README.md)
- [Examples](./examples/) - Ready-to-use configuration examples
- [JSON Schema](./schema.json) - Configuration file validation schema

## 💡 Use Cases

- **Frontend Development**: Mock backend APIs and serve static assets during frontend development
- **API Testing**: Create predictable responses for automated tests
- **Prototyping**: Quickly prototype API designs with static assets without full backend implementation
- **Integration Testing**: Mock external services in integration test suites
- **Demo Applications**: Provide realistic data and serve demo assets for presentations
- **Development Workflows**: Support offline development and testing with both API and static content
- **Asset Testing**: Test static asset delivery alongside API functionality

## 📞 Support

- **Documentation**: Check the [Core Library README](./crates/core/README.md) for detailed API documentation
- **Examples**: Check the [examples directory](./examples/) for ready-to-use configurations
- **Schema**: Use the [JSON schema](./schema.json) for configuration validation in your editor

---

**JSON Echo** - Making API mocking simple, powerful, and developer-friendly.
