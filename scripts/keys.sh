for key in $(etcdctl get --prefix --keys-only /); do
	size=$(etcdctl get $key --print-value-only | wc -c)
	count=$(etcdctl get $key --write-out=fields | grep \"Count\" | cut -f2 -d':')
	if [ $count -ne 0 ]; then
		versions=$(etcdctl get $key --write-out=fields | grep \"Version\" | cut -f2 -d':')
	else
		versions=0
	fi
	total=$(($size * $versions))
	echo $total $size $versions $count $key >>/tmp/etcdkeys.txt
done
