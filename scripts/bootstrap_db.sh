#!/bin/bash

PASSWORD=`head -c1000 /dev/urandom | tr -dc [:alpha:][:digit:] | head -c 16; echo ;`
JWT_SECRET=`head -c1000 /dev/urandom | tr -dc [:alpha:][:digit:] | head -c 32; echo ;`
SECRET_KEY=`head -c1000 /dev/urandom | tr -dc [:alpha:][:digit:] | head -c 32; echo ;`
DB=rust_auth_server
DOMAIN=localhost
SENDING_EMAIL_ADDRESS="user@localhost"
CALLBACK_URL="https://localhost/auth/register.html"

sudo apt-get install -y postgresql

sudo -u postgres createuser -E -e $USER
sudo -u postgres psql -c "CREATE ROLE $USER PASSWORD '$PASSWORD' NOSUPERUSER NOCREATEDB NOCREATEROLE INHERIT LOGIN;"
sudo -u postgres psql -c "ALTER ROLE $USER PASSWORD '$PASSWORD' NOSUPERUSER NOCREATEDB NOCREATEROLE INHERIT LOGIN;"
sudo -u postgres createdb $DB

mkdir -p ${HOME}/.config/aws_app_rust
cat > ${HOME}/.config/sync_app_rust/config.env <<EOL
DOMAIN=$DOMAIN
SENDING_EMAIL_ADDRESS=$SENDING_EMAIL_ADDRESS
CALLBACK_URL=$CALLBACK_URL
AUTHDB=postgresql://$USER:$PASSWORD@localhost:5432/$DB
JWT_SECRET=$JWT_SECRET
SECRET_KEY=$SECRET_KEY
EOL
