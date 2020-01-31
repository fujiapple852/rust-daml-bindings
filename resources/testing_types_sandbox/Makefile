build:
	daml build --target 1.7

inspect:
	daml damlc inspect-dar .daml/dist/*

run-wallclock:
	nohup daml sandbox --port 8080 -w --ledgerid sandbox-wallclock .daml/dist/* >sandbox_wallclock.log 2>&1 &

run-static:
	nohup daml sandbox --port 8081 --ledgerid sandbox-static .daml/dist/* >sandbox_static.log 2>&1 &

run-wallclock-hs256-unsafe:
	nohup daml sandbox --port 8080 -w --ledgerid sandbox-wallclock --auth-jwt-hs256-unsafe testsecret .daml/dist/* >sandbox_wallclock.log 2>&1 &

run-static-hs256-unsafe:
	nohup daml sandbox --port 8081 --ledgerid sandbox-static --auth-jwt-hs256-unsafe testsecret .daml/dist/* >sandbox_static.log 2>&1 &

run-wallclock-rs256:
	nohup daml sandbox --port 8080 -w --ledgerid sandbox-wallclock --auth-jwt-rs256-crt certs/rs256.cert .daml/dist/* >sandbox_wallclock.log 2>&1 &

run-static-rs256:
	nohup daml sandbox --port 8081 --ledgerid sandbox-static --auth-jwt-rs256-crt certs/rs256.cert .daml/dist/* >sandbox_static.log 2>&1 &

run-wallclock-ec256:
	nohup daml sandbox --port 8080 -w --ledgerid sandbox-wallclock --auth-jwt-ec256-crt certs/ec256.cert .daml/dist/* >sandbox_wallclock.log 2>&1 &

run-static-ec256:
	nohup daml sandbox --port 8081 --ledgerid sandbox-static --auth-jwt-ec256-crt certs/ec256.cert .daml/dist/* >sandbox_static.log 2>&1 &

run-wallclock-db:
	nohup daml sandbox --port 8080 --ledgerid sandbox-wallclock -w --jdbcurl "jdbc:postgresql://localhost:54320/wallclock_sandbox?user=postgres" .daml/dist/* &

run-static-db:
	nohup daml sandbox --port 8081 --ledgerid sandbox-static --jdbcurl "jdbc:postgresql://localhost:54320/static_sandbox?user=postgres" .daml/dist/* &

run-navigator-wallclock:
	daml navigator server localhost 8080

run-navigator-static:
	daml navigator server localhost 8081

run: build run-wallclock run-static

run-db: build run-wallclock-db run-static-db

run-hs256: build run-wallclock-hs256-unsafe run-static-hs256-unsafe

run-rs256: build gen-certs run-wallclock-rs256 run-static-rs256

run-ec256: build gen-certs run-wallclock-ec256 run-static-ec256

run-only: run-wallclock run-static

restart: kill run-only

kill:
	jps | grep daml-sdk | cut -d" " -f 1 | xargs kill

show:
	jps | grep daml-sdk

db-start:
	docker run --rm -d --name my_postgres -v my_dbdata:/var/lib/postgresql/data -p 54320:5432 postgres:11

db-stop:
	docker stop my_postgres

db-table-create:
	docker exec -it my_postgres psql -U postgres -c "create database static_sandbox"
	docker exec -it my_postgres psql -U postgres -c "create database wallclock_sandbox"

db-table-drop:
	docker exec -it my_postgres psql -U postgres -c "drop database static_sandbox"
	docker exec -it my_postgres psql -U postgres -c "drop database wallclock_sandbox"

db-sql:
	docker exec -it my_postgres psql -U postgres -d my_database

gen-certs:
	./generate_certs.sh