# Deployment Guide

This guide covers deploying the Stressor Leads application to Render (backend) and Vercel (frontend).

## Backend Deployment (Render)

### Prerequisites
- GitHub account with the repository pushed
- Render account (free tier available)

### Steps

1. **Go to Render Dashboard**: https://dashboard.render.com

2. **Create a New Web Service**:
   - Click "New +" → "Web Service"
   - Connect your GitHub repository
   - Select the `stressor-leads` repository

3. **Configure the Service**:
   - **Name**: `stressor-leads-api`
   - **Environment**: `Rust`
   - **Build Command**: `cargo build --release`
   - **Start Command**: `./target/release/stressor-leads`
   - **Plan**: Free (or paid for better performance)

4. **Create PostgreSQL Database**:
   - Click "New +" → "PostgreSQL"
   - Name: `stressor-leads-db`
   - Plan: Free (or paid)
   - Note the connection string

5. **Set Environment Variables**:
   - `DATABASE_URL`: Your PostgreSQL connection string from step 4
   - `JWT_SECRET`: Generate a secure random string (e.g., `openssl rand -hex 32`)
   - `PORT`: `10000` (Render uses port 10000)
   - `FRONTEND_URL`: Your Vercel frontend URL (set after deploying frontend)

6. **Deploy**:
   - Click "Create Web Service"
   - Render will build and deploy your application
   - Note the service URL (e.g., `https://stressor-leads-api.onrender.com`)

### Health Check
Your API will be available at: `https://your-service.onrender.com/api/health`

## Frontend Deployment (Vercel)

### Prerequisites
- GitHub account with the frontend repository pushed
- Vercel account (free tier available)

### Steps

1. **Go to Vercel Dashboard**: https://vercel.com/dashboard

2. **Import Project**:
   - Click "Add New..." → "Project"
   - Import your GitHub repository (`stressor-leads-frontend`)
   - Or create a new repository and push the frontend code

3. **Configure Project**:
   - **Framework Preset**: Next.js (auto-detected)
   - **Root Directory**: `./` (or `stressor-leads-frontend` if in monorepo)
   - **Build Command**: `npm run build` (default)
   - **Output Directory**: `.next` (default)

4. **Set Environment Variables**:
   - `NEXT_PUBLIC_API_URL`: Your Render API URL (e.g., `https://stressor-leads-api.onrender.com`)

5. **Deploy**:
   - Click "Deploy"
   - Vercel will build and deploy your application
   - Note the deployment URL (e.g., `https://stressor-leads-frontend.vercel.app`)

6. **Update Backend CORS**:
   - Go back to Render dashboard
   - Update `FRONTEND_URL` environment variable to your Vercel URL
   - Redeploy the backend service

## Post-Deployment

### Testing

1. **Test Backend**:
   ```bash
   curl https://your-api.onrender.com/api/health
   ```

2. **Test Frontend**:
   - Visit your Vercel URL
   - Register a new account
   - Upload a CSV file
   - View scored leads

### CSV File Format

Create a CSV file with the following columns:
```csv
vin,warranty_exp_date,customer_name,customer_phone,customer_email,customer_zip,last_service_date
1HGBH41JXMN109186,2025-12-31,John Doe,555-1234,john@example.com,12345,2024-01-15
```

### Monitoring

- **Render**: Check logs in the Render dashboard
- **Vercel**: Check logs in the Vercel dashboard
- **Database**: Monitor in Render PostgreSQL dashboard

## Troubleshooting

### Backend Issues

- **Build fails**: Check Rust version compatibility
- **Database connection fails**: Verify `DATABASE_URL` is correct
- **CORS errors**: Ensure `FRONTEND_URL` matches your Vercel URL exactly

### Frontend Issues

- **API calls fail**: Verify `NEXT_PUBLIC_API_URL` is set correctly
- **Build fails**: Check Node.js version (should be 18+)
- **Authentication issues**: Verify JWT token is being stored in localStorage

## Environment Variables Summary

### Backend (Render)
- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: Secret key for JWT tokens
- `PORT`: `10000` (Render default)
- `FRONTEND_URL`: Your Vercel frontend URL

### Frontend (Vercel)
- `NEXT_PUBLIC_API_URL`: Your Render API URL

## Cost

- **Render Free Tier**: 
  - Web service spins down after 15 minutes of inactivity
  - PostgreSQL database (limited storage)
- **Vercel Free Tier**:
  - Unlimited deployments
  - 100GB bandwidth/month
  - Perfect for development and small projects

For production use, consider upgrading to paid plans for better performance and reliability.

