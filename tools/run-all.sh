for quality in normal acceptable degraded horrible; do
  for i in 1 2 3 4 5; do
    ./do-run.sh $quality iot udp $i
  done
done
