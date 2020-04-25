containers=(
  proxy
  producer
  consumer
)

for name in "${containers[@]}"; do
  echo "Stopping $name..."
  docker stop $name
  echo "Removing $name..."
  docker rm $name
done
