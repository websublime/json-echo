# JSON Echo Examples

This directory contains ready-to-use configuration examples demonstrating various features of JSON Echo. Each example showcases different aspects of the mock API server functionality.

## 📁 Available Examples

### 1. `basic-api.json`
**A simple API configuration with inline responses**

```bash
echo --config examples/basic-api.json serve
```

**Features demonstrated:**
- Inline JSON responses
- Basic route configuration
- Parameterized routes with ID lookup
- Health check endpoint

**Available endpoints:**
- `GET /api/users` - List all users
- `GET /api/users/{id}` - Get user by ID
- `GET /api/health` - Health check

### 2. `external-files.json`
**Configuration using external JSON files for responses**

```bash
echo --config examples/external-files.json serve
```

**Features demonstrated:**
- External file references for response data
- Custom ID fields (`product_id`, `order_id`)
- Results field nesting
- Cross-platform file loading

**Available endpoints:**
- `GET /api/products` - Product catalog from external file
- `GET /api/products/{id}` - Get product by ID
- `GET /api/categories` - Categories from external file
- `GET /api/orders` - Orders from external file
- `GET /api/orders/{id}` - Get order by ID

### 3. `static-files.json`
**Configuration with static file serving for assets and frontend files**

```bash
echo --config examples/static-files.json serve
```

**Features demonstrated:**
- Static file serving configuration
- Mixed API and static content
- Asset references in API responses
- File upload mock endpoints
- Avatar and image serving

**Available endpoints:**
- `GET /api/health` - Health check with version info
- `GET /api/config` - App configuration with static URLs
- `GET /api/assets/manifest` - Asset manifest for builds
- `GET /api/users` - Users with avatar URLs
- `GET /api/users/{id}` - User details with profile assets
- `GET /api/posts` - Blog posts with featured images
- `POST /api/media/upload` - Mock file upload
- `GET /static/*` - Static files (HTML, CSS, JS, images)

### 4. `full-featured.json`
**Comprehensive example showcasing all JSON Echo features**

```bash
echo --config examples/full-featured.json serve
```

**Features demonstrated:**
- Mixed inline and external file responses
- Custom headers configuration
- Different HTTP status codes (200, 404, 500, 503)
- Complex nested data structures
- Custom ID fields and results fields
- Error response examples
- Analytics and dashboard data
- Pagination and metadata

**Available endpoints:**
- `GET /api/health` - Enhanced health check
- `GET /api/users` - Users with pagination
- `GET /api/users/{id}` - User details with permissions
- `GET /api/products` - Products from external file
- `GET /api/products/{code}` - Product by SKU code
- `GET /api/categories` - Categories with nested structure
- `GET /api/orders` - Orders with complex data
- `GET /api/orders/{id}` - Order details
- `GET /api/analytics/dashboard` - Dashboard metrics
- `GET /api/status` - Maintenance status (503)
- `GET /api/errors/not-found` - Example 404 error
- `GET /api/errors/server-error` - Example 500 error

## 🧪 Testing the Examples

### Start a Server

```bash
# Basic API
echo --config examples/basic-api.json serve

# External files (requires data files)
echo --config examples/external-files.json serve

# Static file serving
echo --config examples/static-files.json serve

# Full featured
echo --config examples/full-featured.json serve
```

### Test Endpoints

```bash
# Health check
curl http://localhost:3001/api/health

# List users
curl http://localhost:3001/api/users

# Get specific user
curl http://localhost:3001/api/users/1

# Get products (external file example)
curl http://localhost:8080/api/products

# Get product by custom field
curl http://localhost:3001/api/products/MBP-16-M2MAX-512

# Test static files
curl http://localhost:3001/static/index.html
curl http://localhost:3001/static/css/style.css
curl http://localhost:3001/static/images/logo.png

# Test error responses
curl http://localhost:3001/api/errors/not-found
curl http://localhost:3001/api/status
```

### CORS Testing

```bash
# Test CORS preflight
curl -X OPTIONS http://localhost:3001/api/users \
  -H "Origin: http://localhost:3000" \
  -H "Access-Control-Request-Method: GET" \
  -v

# Test actual CORS request
curl -X GET http://localhost:3001/api/users \
  -H "Origin: http://localhost:3000" \
  -v

# Test CORS with static files
curl -X OPTIONS http://localhost:3001/static/app.js \
  -H "Origin: http://localhost:3000" \
  -H "Access-Control-Request-Method: GET" \
  -v
```

## 📊 Data Files

The `data/` directory contains sample JSON files used by the external file examples:

### `products.json`
Product catalog with:
- 6 sample products (laptops, smartphones, audio, accessories)
- Complete product specifications
- Stock information and pricing
- Metadata with pagination info

### `categories.json`
Product categories featuring:
- Hierarchical category structure
- Parent/child relationships
- SEO metadata
- Category statistics

## 🚀 Quick Start Guide

1. **Choose an example** based on your needs:
   - New to JSON Echo? Start with `basic-api.json`
   - Need external files? Try `external-files.json`
   - Want static file serving? Use `static-files.json`
   - Building a SPA? Try `spa-config.json`
   - Want to see everything? Use `full-featured.json`

2. **Start the server:**
   ```bash
   echo --config examples/[chosen-example].json serve
   ```

3. **Test with curl or your favorite HTTP client:**
   ```bash
   curl http://localhost:3001/api/health
   ```

4. **Customize for your needs:**
   - Copy an example to your project
   - Modify routes and responses
   - Add your own data files

## 🔧 Configuration Tips

### Custom Ports and Hosts
```bash
# Run on different port
echo --config examples/basic-api.json serve
# Then edit the "port" field in the JSON file

# Run on all interfaces
# Set "hostname": "0.0.0.0" in the configuration
```

### Debug Mode
```bash
echo --log-level debug --config examples/full-featured.json serve
```

### Custom Configuration File Paths
```bash
# Absolute path
echo --config /path/to/your/config.json serve

# Relative to current directory
echo --config ./my-custom-config.json serve
```

## 📝 Creating Your Own Examples

1. **Copy an existing example:**
   ```bash
   cp examples/basic-api.json my-api.json
   ```

2. **Modify the configuration:**
   - Update routes to match your API design
   - Change response data to match your use case
   - Adjust ports and hostnames as needed

3. **Test your configuration:**
   ```bash
   echo --config my-api.json serve
   ```

4. **Validate with curl:**
   ```bash
   curl http://localhost:3001/your-endpoint
   ```

## 🎯 Use Case Examples

### Frontend Development
Use `basic-api.json` to quickly mock your backend API during frontend development:

```bash
echo --config examples/basic-api.json serve
# Your frontend can now call http://localhost:3001/api/users
```

### Static Asset Testing
Use `static-files.json` to test both API and static content delivery:

```bash
echo --config examples/static-files.json serve
# API: http://localhost:3001/api/users
# Assets: http://localhost:3001/static/css/style.css
```

### API Testing
Use `full-featured.json` to test error handling and edge cases:

```bash
echo --config examples/full-featured.json serve
# Test 404: curl http://localhost:3001/api/errors/not-found
# Test 500: curl http://localhost:3001/api/errors/server-error
```

### Demo and Prototyping
Use `external-files.json` with rich data for demos:

```bash
echo --config examples/external-files.json serve
# Rich product catalog at http://localhost:8080/api/products
```

## 🔗 Related Documentation

- [Main README](../README.md) - Complete JSON Echo documentation
- [Core Library](../crates/core/README.md) - Detailed API documentation
- [JSON Schema](../schema.json) - Configuration validation schema

## 💡 Tips and Best Practices

1. **Start Simple**: Begin with `basic-api.json` and gradually add complexity
2. **Use External Files**: For large datasets, use external JSON files like in `external-files.json`
3. **Combine API and Static Content**: Use `static-files.json` for mixed API and asset serving
4. **Test Different Status Codes**: Use examples from `full-featured.json` to test error handling
5. **Organize Data**: Keep data files in a separate directory for better organization
6. **Document Your API**: Add descriptions to all routes for better maintainability
7. **Use Meaningful IDs**: Choose ID fields that make sense for your data model
8. **Plan for Pagination**: Structure your responses to support pagination from the start
9. **Static File Organization**: Structure your static files logically (css/, js/, images/)

Happy mocking! 🚀
