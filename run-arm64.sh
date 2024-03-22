docker compose -f docker-compose.arm64.yml rm -f
docker compose -f docker-compose.arm64.yml down
docker compose -f docker-compose.arm64.yml up --build