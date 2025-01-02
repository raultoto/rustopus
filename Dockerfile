FROM rust:1.74-slim as builder

WORKDIR /app
COPY . .

# Build the application in release mode
RUN cargo build --release

# Create a new stage with a minimal image
FROM debian:bookworm-slim

# Create necessary directories
RUN mkdir -p /etc/rustopus /var/log/rustopus

WORKDIR /app

# Copy the built executable
COPY --from=builder /app/target/release/rustopus /usr/local/bin/rustopus
# Copy default configuration files
COPY config.yaml /etc/rustopus/config.yaml.template
COPY config.json /etc/rustopus/config.json.template
COPY schema.json /etc/rustopus/schema.json

# Make the binary executable
RUN chmod +x /usr/local/bin/rustopus

# Create a non-root user
RUN useradd -r -s /bin/false rustopus && \
    chown -R rustopus:rustopus /etc/rustopus /var/log/rustopus

USER rustopus

# Expose the default port
EXPOSE 8080

# Set default config path
ENV CONFIG_PATH=/etc/rustopus/config.yaml

# Entrypoint script to allow configuration override
ENTRYPOINT ["rustopus"]
CMD ["--config", "/etc/rustopus/config.yaml"] 