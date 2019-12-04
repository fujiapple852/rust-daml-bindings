build:
	daml build --target 1.7

inspect:
	daml damlc inspect-dar .daml/dist/*

run-wallclock:
	daml sandbox --port 8080 -w .daml/dist/* &

run-static:
	daml sandbox --port 8081 .daml/dist/* &

run-wallclock-db:
	daml sandbox --port 8080 --ledgerid sandbox-wallclock -w --jdbcurl "jdbc:postgresql://localhost:54320/wallclock_sandbox?user=postgres" .daml/dist/* &

run-static-db:
	daml sandbox --port 8081 --ledgerid sandbox-static --jdbcurl "jdbc:postgresql://localhost:54320/static_sandbox?user=postgres" .daml/dist/* &

run-navigator-wallclock:
	daml navigator server localhost 8080

run-navigator-static:
	daml navigator server localhost 8081

run: build run-wallclock run-static

run-db: build run-wallclock-db run-static-db

run-only: run-wallclock run-static

restart: kill run-only

kill:
	jps | grep sandbox | cut -d" " -f 1 | xargs kill

show:
	jps | grep sandbox

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

