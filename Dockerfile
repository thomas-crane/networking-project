FROM alpine

# copy the binaries.
COPY bins/* /usr/bin/

# install the `tc` tool.
RUN apk add iproute2
