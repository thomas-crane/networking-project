containers=(
  producer
  consumer
)

for name in "${containers[@]}"; do
  echo "Stopping $name..."
  docker kill $name
  echo "Removing $name..."
  docker rm $name
done
