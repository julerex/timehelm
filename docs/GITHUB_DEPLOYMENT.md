# GitHub Actions Deployment to Fly.io

This guide explains how to set up automated deployment to Fly.io using GitHub Actions.

## Overview

The project includes a GitHub Actions workflow (`.github/workflows/deploy.yml`) that automatically deploys your application to Fly.io whenever you push to the `main` or `master` branch.

## Prerequisites

1. A Fly.io account (sign up at [fly.io](https://fly.io))
2. The Fly.io CLI installed locally
3. Your app already initialized on Fly.io (run `fly launch` or `fly apps create` at least once)

## Setup Steps

### 1. Get Your Fly.io API Token

Run the following command in your terminal:

```bash
flyctl auth token
```

This will output your API token. Copy this token - you'll need it in the next step.

### 2. Add the Token as a GitHub Secret

1. Go to your GitHub repository
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **"New repository secret"**
4. Set the name to: `FLY_API_TOKEN`
5. Paste your API token from step 1 as the value
6. Click **"Add secret"**

### 3. Push the Workflow to GitHub

Commit and push the workflow file to your repository:

```bash
git add .github/workflows/deploy.yml
git commit -m "Add GitHub Actions workflow for Fly.io deployment"
git push origin main
```

## How It Works

- The workflow triggers automatically on pushes to `main` or `master` branches
- It can also be triggered manually via the GitHub Actions tab
- The deployment uses `flyctl deploy --remote-only`, which builds your Docker image on Fly.io's servers
- Your `Dockerfile` handles building both the Rust server and Node.js client

## Manual Deployment

If you need to deploy manually without using GitHub Actions, you can use:

```bash
make deploy
```

Or directly:

```bash
./build.sh
fly deploy
```

## Troubleshooting

- **Workflow fails with authentication error**: Make sure `FLY_API_TOKEN` is set correctly in GitHub Secrets
- **Build fails**: Check that your `Dockerfile` is correct and all dependencies are properly specified
- **Deployment fails**: Ensure your `fly.toml` is configured correctly and the app exists on Fly.io
