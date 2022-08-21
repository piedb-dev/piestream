set -ex

IMAGE='~/image/rise.tar'

docker save piestream > ~/image/rise.tar

if [ -x `which pdsh` ]; then
	pdsh -R exec -l abc -w host[01-04],meta scp ~/image/rise.tar %u@%h:~/image/rise.tar
	pdsh -R ssh  -l abc -w host[01-04],meta "docker load -i ~/image/rise.tar"
else
	scp ~/image/rise.tar host02:~/image/rise.tar
	ssh host02 "docker load -i ~/image/rise.tar"
	scp ~/image/rise.tar host03:~/image/rise.tar
	ssh host03 "docker load -i ~/image/rise.tar"
	scp ~/image/rise.tar host04:~/image/rise.tar
	ssh host04 "docker load -i ~/image/rise.tar"
	scp ~/image/rise.tar meta:~/image/rise.tar
	ssh meta "docker load -i ~/image/rise.tar"
fi
