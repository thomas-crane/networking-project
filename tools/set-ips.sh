#
# this script should be sourced, not run. It
# expects there to be 3 containers with specific names.
#
# https://stackoverflow.com/questions/17157721/how-to-get-a-docker-containers-ip-address-from-the-host
export PROXY_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' proxy)
export CONSUMER_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' consumer)
export PRODUCER_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' producer)
