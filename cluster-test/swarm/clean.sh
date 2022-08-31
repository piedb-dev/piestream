for i in {1..4}; do
	echo "STOPPING HOST0$i"
	ssh host0$i 'echo y | docker volume prune'
done