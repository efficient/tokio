readonly   URL="http://192.168.0.1:1337"

coregroup() {
	local start="$1"
	local count="$2"
	shift 2

	for core in `seq "$start" 2 "$((count * 2 - 1))"`
	do
		if [ "$core" = "$start" ]
		then
			set -x
		fi
		taskset -c "$core" env LD_LIBRARY_PATH=. ./ld-linux-x86-64.so.2 ./wrk -t1 -d30 --latency "$@" &
		set +x
	done
}

logfile() {
	local half=""
	if [ $# -ne 0 ]
	then
		half="$1"
	fi

	echo "${indep}_${series}${half}_$name$const.log"
}

set -eu
cd "`dirname "$0"`"
