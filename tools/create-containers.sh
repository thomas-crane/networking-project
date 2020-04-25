containers=(
  proxy
  producer
  consumer
)
for name in "${containers[@]}"; do
  echo "Creating $name..."
  docker run -itd --cap-add NET_ADMIN --name $name tom/net-project
done
