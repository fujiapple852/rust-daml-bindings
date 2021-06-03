DAML_LF_VERSION = 1.12
OUTPUT_PATH = .daml/dist
DAR_NAME = TestingTypes

SANDBOX_HOST = 127.0.0.1
WALLCLOCK_PORT = 8080
STATIC_PORT = 8081
WALLCLOCK_UNSECURED_PORT = 8085
JSON_BRIDGE_PORT = 7575

SDK_VERSION := $(shell daml version | grep project | cut -d" " -f 3)
DAR_VERSION := $(shell egrep "^version" daml.yaml | cut -d" " -f 2)
OUTPUT_DAR_NAME := $(shell echo ${DAR_NAME}-${DAR_VERSION}-sdk_${SDK_VERSION}-lf_${DAML_LF_VERSION} | sed "s/\./_/g")
OUTPUT_FULL = ${OUTPUT_PATH}/${OUTPUT_DAR_NAME}.dar

.PHONY: build archive

build:
	rm -rf ${OUTPUT_PATH}
	daml build --target ${DAML_LF_VERSION} --output ${OUTPUT_FULL}

archive:
	cp ${OUTPUT_PATH}/${OUTPUT_DAR_NAME}.dar archive

archive-latest: archive
	ln -sf archive/${OUTPUT_DAR_NAME}.dar ${DAR_NAME}-latest.dar

inspect:
	daml damlc inspect-dar .daml/dist/*

run-wallclock:
	./sandbox.sh -h ${SANDBOX_HOST} -p ${WALLCLOCK_PORT} -l wallclock-sandbox -m wallclock -a es256

run-static:
	./sandbox.sh -h ${SANDBOX_HOST} -p ${STATIC_PORT} -l static-sandbox -m static -a es256

run-wallclock-unsecured:
	./sandbox.sh -h ${SANDBOX_HOST} -p ${WALLCLOCK_UNSECURED_PORT} -l wallclock-unsecured-sandbox -m wallclock

run-json-bridge:
	./json_bridge.sh -h ${SANDBOX_HOST} -p ${WALLCLOCK_UNSECURED_PORT} -b ${JSON_BRIDGE_PORT}

run-all: build run-wallclock-unsecured gen-certs run-wallclock run-static run-json-bridge

run: kill run-all

run-ci: run-all

kill:
	jps | grep daml-sdk | cut -d" " -f 1 | xargs kill
	ps | grep json_bridge | grep -v grep | cut -d" " -f 1 | xargs kill

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