# godaddy-cli

Simple godaddy API wrapper for managing GoDaddy DNS records.

## Prerequisites

You should have an API key and secret for your GoDaddy account. You can create one [here](https://developer.godaddy.com/keys).
Put the key and secret in a file called `godaddy.conf` in the same directory as the CLI.

```
your-key：your-secret
```

Or you can set the environment variables `API_KEY`to the `your-key：your-secret`

## Usage

### Interactive mode

```
./godaddy-cli
```

Choose operations from the menu.

### Non-interactive mode

By providing flags, you can run the CLI in non-interactive mode.

1. List all domains

```
./godaddy-cli list
```

2. List all records for a domain

```
./godaddy-cli records example.com
```

Where `example.com` is the domain name you have under godaddy account.

3. Add a record under a domain

```
./godaddy-cli add example.com --name aaa --ip 1.2.3.4
```

This will add a record with name `aaa` and with ip 1.2.3.4 under domain `example.com`. You should already have domain `example.com` under your godaddy account.

If --name is not provided, it will generate a random english word for the subdomain.
If --ip is not provided, it will use your current public ip address.

4. Wait for a record to be ready

```
./godaddy-cli add example.com --name aaa --ip 1.2.3.4 --wait
```

The --wait flag can be provided to wait until the record is propagated to DNS servers.
