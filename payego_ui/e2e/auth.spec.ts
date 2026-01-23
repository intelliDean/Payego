import { test, expect } from '@playwright/test';

test.describe('Authentication Flow', () => {
    test('should allow a user to register and login', async ({ page }) => {
        // 1. Generate unique user
        const timestamp = Date.now();
        const email = `testuser${timestamp}@example.com`;
        const password = 'password123';
        const firstName = 'Test';
        const lastName = 'User';

        // 2. Go to Register page
        await page.goto('/register');
        await expect(page).toHaveURL(/.*register/);

        // 3. Fill Registration Form
        await page.fill('input[name="firstName"]', firstName);
        await page.fill('input[name="lastName"]', lastName);
        await page.fill('input[name="email"]', email);
        await page.fill('input[name="password"]', password);
        await page.fill('input[name="confirmPassword"]', password);

        // 4. Submit
        await page.click('button[type="submit"]');

        // 5. Expect redirect to Login (or Dashboard depending on flow, usually Login)
        await expect(page).toHaveURL(/.*login/);

        // 6. Login
        await page.fill('input[name="email"]', email);
        await page.fill('input[name="password"]', password);
        await page.click('button[type="submit"]');

        // 7. Verify Dashboard
        await expect(page).toHaveURL(/.*dashboard/);
        await expect(page.locator('h1')).toContainText('Dashboard');
    });
});
