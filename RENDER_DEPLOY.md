# Render Deployment Guide

## Quick Deploy

You can deploy using the automated script:

```bash
cd stressor-leads
./deploy-render.sh
```

Or manually through the Render dashboard:

## Manual Deployment Steps

### 1. Create PostgreSQL Database

1. Go to https://dashboard.render.com
2. Click "New +" → "PostgreSQL"
3. Configure:
   - **Name**: `stressor-leads-db`
   - **Database**: `stressor_leads`
   - **User**: `stressor_leads`
   - **Plan**: Free
   - **Region**: Oregon (or closest to you)
4. Click "Create Database"
5. **Copy the Internal Database URL** (you'll need this)

### 2. Create Web Service

1. Go to https://dashboard.render.com
2. Click "New +" → "Web Service"
3. Connect GitHub repository: `beargallbladder/V1PROGDEMO`
4. Configure:
   - **Name**: `stressor-leads-api`
   - **Environment**: `Rust`
   - **Region**: Oregon (or closest to you)
   - **Branch**: `main`
   - **Root Directory**: `stressor-leads`
   - **Build Command**: `cargo build --release`
   - **Start Command**: `./target/release/stressor-leads`
   - **Plan**: Free

### 3. Set Environment Variables

In the Render dashboard, add these environment variables:

- **DATABASE_URL**: (Internal Database URL from step 1)
- **JWT_SECRET**: Generate with `openssl rand -hex 32`
- **PORT**: `10000`
- **FRONTEND_URL**: (Set after deploying frontend to Vercel)

### 4. Deploy

Click "Create Web Service" and wait for deployment to complete.

### 5. Get Your API URL

After deployment, your API will be available at:
`https://stressor-leads-api.onrender.com`

Test it:
```bash
curl https://stressor-leads-api.onrender.com/api/health
```

## Using Render API (Alternative)

You can also use the Render API directly. The script `deploy-render.sh` automates this process.

## Post-Deployment

1. **Test the API**:
   ```bash
   curl https://your-api.onrender.com/api/health
   ```

2. **Deploy Frontend to Vercel** (see frontend README)

3. **Update CORS**:
   - Add `FRONTEND_URL` environment variable in Render
   - Set it to your Vercel frontend URL
   - Redeploy the service

## Troubleshooting

- **Build fails**: Check logs in Render dashboard
- **Database connection fails**: Verify `DATABASE_URL` is correct
- **Service won't start**: Check `PORT` is set to `10000`
- **CORS errors**: Ensure `FRONTEND_URL` matches your Vercel URL

## Free Tier Limitations

- Services spin down after 15 minutes of inactivity
- First request after spin-down may take 30-60 seconds
- Database has limited storage (1GB on free tier)

For production, consider upgrading to paid plans.

