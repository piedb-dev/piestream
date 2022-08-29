for i in {1..4}; do
	echo "STARTING HOST0$i"
	ssh host0$i 'cd rise; docker-compose -f node'$i'.docker-compose.yml up -d' &> /dev/null &
done
docker-compose -f meta.docker-compose.yml up -d 
