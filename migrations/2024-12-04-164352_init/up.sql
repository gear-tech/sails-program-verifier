-- Your SQL goes here
CREATE TYPE VERIFICATIONSTATUS AS ENUM ('pending', 'in_progress', 'failed', 'verified');

CREATE TYPE NETWORK AS ENUM ('vara_mainnet', 'vara_testnet');

CREATE TABLE "idl" (
    "id" VARCHAR NOT NULL PRIMARY KEY,
    "content" TEXT NOT NULL
);

CREATE TABLE "verification" (
    "id" VARCHAR NOT NULL PRIMARY KEY,
    "repo_link" VARCHAR NOT NULL,
    "code_id" VARCHAR NOT NULL,
    "project_name" VARCHAR,
    "build_idl" BOOL NOT NULL,
    "version" VARCHAR NOT NULL,
    "status" VERIFICATIONSTATUS NOT NULL,
    "network" NETWORK NOT NULL,
    "failed_reason" TEXT,
    "created_at" TIMESTAMP NOT NULL
);

CREATE TABLE "code" (
    "id" VARCHAR NOT NULL PRIMARY KEY,
    "idl_hash" VARCHAR,
    "name" VARCHAR NOT NULL,
    "repo_link" VARCHAR NOT NULL
);
