.PHONY: infra dev stop clean backend webapp

# Start the Docker Compose infrastructure in the background
infra:
	docker-compose up -d

# Stop the Docker Compose infrastructure
stop:
	docker-compose down

# Clean up Docker volumes (wipes database and redis)
clean:
	docker-compose down -v

# Run the Axum backend with hot-reloading
# Requires: cargo install cargo-watch
backend:
	cd axum_slack_proxy && cargo watch -q -c -w src/ -x run

# Run the Vite Web App natively
webapp:
	cd web-app && npm run dev

# The ultimate rapid dev command: Starts infra, backend, and web app
dev: infra
	@echo "Infrastructure is running. Booting native servers..."
	@echo "To stop, press Ctrl+C and run 'make stop'"
	# We use basic bash to run both natively in parallel
	(make backend & make webapp & wait)
