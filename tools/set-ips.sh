#
# this script should be sourced, not run. It
# expects there to be 3 containers with specific names.
#
# https://stackoverflow.com/questions/17157721/how-to-get-a-docker-containers-ip-address-from-the-host
export CONSUMER_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' consumer)
