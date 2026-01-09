# Connecting Your Fly App to Managed Fly Postgres

## 1. Attach Your App to the Postgres Cluster

**Using the CLI** (recommended):

```bash
fly mpg attach <cluster-id> -a <your-app-name>
```

Replace `<cluster-id>` with your Postgres cluster ID (e.g., `timehelm-db`) and `<your-app-name>` with your Fly app name.

**Or via the Dashboard:**

1. Go to [fly.io/dashboard](https://fly.io/dashboard)
2. Select **Managed Postgres** → your cluster → **Connect** tab
3. In "Attach an App", select your app
4. Click **Set secret and deploy**

## 2. What This Does

The `attach` command:

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
| `fly mpg list`                         | List your managed Postgres clusters  |
| `fly mpg connect <cluster-id>`         | Open a psql shell to your database   |
| `fly mpg attach <cluster-id> -a <app>` | Attach an app                        |
| `fly mpg detach <cluster-id> -a <app>` | Detach an app                        |

