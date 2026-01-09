# Connecting Your Fly App to Managed Fly Postgres

## 1. Attach Your App to the Postgres Cluster

### Using the CLI** (recommended):

```bash
fly mpg attach <cluster-id> -a <your-app-name>
```

Replace `<cluster-id>` with your Managed Postgres cluster ID and `<your-app-name>` with your Fly app name.

- When prompted, select **Create new user...**
- When prompted, enter the desired **username**
- When prompted, select **writer** for the **user role**
- When prompted, select **Create new database...**
- When prompted, enter the desired **database name**



#### Specifying a database name:**

By default, a database is created with the same name as your app. To use a specific database name, add the `--database` flag:

```bash
fly mpg attach <cluster-id> -a <your-app-name> --database <database-name>
```


### Or via the Dashboard:

1. Go to [fly.io/dashboard](https://fly.io/dashboard)
2. Select **Managed Postgres** → your cluster → **Connect** tab
3. In "Attach an App", select your app
4. Click **Set secret and deploy**

## 2. What This Does

The `attach` command:

- Creates a database (if it doesn't exist) with the specified name or your app name
- Creates a database user for your app
- Sets the `DATABASE_URL` secret on your app with the full connection string
- Triggers a deployment

## 3. Update Your App to Use `DATABASE_URL`

Make sure your app reads from the `DATABASE_URL` environment variable:

```rust
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
let pool = create_pool(&database_url).await?;
```

## 4. Verify the Connection

Check that the secret was set:

```bash
fly secrets list -a <your-app-name>
```

You should see `DATABASE_URL` in the list.

## Useful Commands

| Command                                | Description                          |
| -------------------------------------- | ------------------------------------ |
| `fly apps list`                        | List your Fly apps                   |
| `fly mpg list`                         | List your managed Postgres clusters  |
| `fly mpg connect <cluster-id>`         | Open a psql shell to your database   |
| `fly mpg attach <cluster-id> -a <app>` | Attach an app                        |
| `fly mpg detach <cluster-id> -a <app>` | Detach an app                        |

