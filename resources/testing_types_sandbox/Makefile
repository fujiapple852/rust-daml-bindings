DAML_LF_VERSION = 1.8

SANDBOX_HOST = 127.0.0.1
WALLCLOCK_PORT = 8080
STATIC_PORT = 8081

build:
	daml build --target ${DAML_LF_VERSION}

inspect:
	daml damlc inspect-dar .daml/dist/*

run-wallclock:
	./sandbox.sh -h ${SANDBOX_HOST} -p ${WALLCLOCK_PORT} -m wallclock -a es256

run-static:
	./sandbox.sh -h ${SANDBOX_HOST} -p ${STATIC_PORT} -m static -a es256

run-all: build gen-certs run-wallclock run-static

run: kill run-all

run-ci: run-all

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