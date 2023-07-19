import { test, expect, Page } from '@playwright/test';

// This should really be able to be any valid feed since we do all of our tests
// with dates relative to a reference start.
const FEED_URL = 'http://feeds.feedburner.com/radiolab';
const ENCODED_FEED_URL = encodeURIComponent(FEED_URL);

test('Happy path', async ({ page, context, browserName }) => {
  await page.goto('/');
  await page.waitForTimeout(200);

  const search = await page.getByPlaceholder(/Enter a URL like/);
  await search.fill(FEED_URL);
  await search.press('Enter');

  await page.getByLabel('Starting').fill('2023-07-01T01:30');

  await expect(page).toHaveURL(
    `/preview?start=2023-07-01T01:30:00-0400&rule=1w&uri=${ENCODED_FEED_URL}`,
  );

  const newPagePromise = context.waitForEvent('page');
  await page.getByText('Subscribe').click();
  const newPage = await newPagePromise;
  await expect(newPage).toHaveURL(
    `/replay?start=2023-07-01T01:30:00-0400&rule=1w&uri=${ENCODED_FEED_URL}`,
  );
  await newPage.close();

  if (browserName !== 'firefox') {
    // Firefox does not support the clipboard.readText API
    // so we're skipping until Playwright adds a clipboard API
    // https://github.com/microsoft/playwright/issues/15860
    await page.getByText('Copy Feed URL').click();
    const clipboard = await page.evaluate('navigator.clipboard.readText()');
    await expect(clipboard).toContain(
      `/replay?start=2023-07-01T01:30:00-0400&rule=1w&uri=${ENCODED_FEED_URL}`,
    );
  }

  expect(await getRescheduledDates(page)).toEqual([
    'Jul 1st, 2023',
    'Jul 8th, 2023',
    'Jul 15th, 2023',
  ]);

  await page.getByLabel(/Two/).click();
  expect(await getRescheduledDates(page)).toEqual([
    'Jul 1st, 2023',
    'Jul 15th, 2023',
    'Jul 29th, 2023',
  ]);
  await expect(page.getByText('Subscribe')).toHaveAttribute(
    'href',
    `http://localhost:3000/replay?start=2023-07-01T01:30:00-0400&rule=2w&uri=${ENCODED_FEED_URL}`,
  );

  await page.getByLabel(/Tue/).click();
  await page.getByLabel(/Thu/).click();
  expect(await getRescheduledDates(page)).toEqual([
    'Jul 4th, 2023',
    'Jul 6th, 2023',
    'Jul 18th, 2023',
  ]);
  await expect(page.getByText('Subscribe')).toHaveAttribute(
    'href',
    `http://localhost:3000/replay?start=2023-07-01T01:30:00-0400&rule=2wTuTh&uri=${ENCODED_FEED_URL}`,
  );

  await page.getByLabel(/Tue/).click();
  await page.getByLabel(/Thu/).click();
  await page.getByLabel(/Days/).click();
  expect(await getRescheduledDates(page)).toEqual([
    'Jul 1st, 2023',
    'Jul 3rd, 2023',
    'Jul 5th, 2023',
  ]);
  await expect(page.getByText('Subscribe')).toHaveAttribute(
    'href',
    `http://localhost:3000/replay?start=2023-07-01T01:30:00-0400&rule=2d&uri=${ENCODED_FEED_URL}`,
  );

  await page.getByLabel('Start here').nth(2).click();
  await page.getByLabel('End here').nth(1).click();
  expect(await getRescheduledDates(page, 5)).toEqual([
    'Skip',
    'Skip',
    'Jul 1st, 2023',
    'Jul 3rd, 2023',
    'Skip',
  ]);
  await expect(page.getByText('Subscribe')).toHaveAttribute(
    'href',
    `http://localhost:3000/replay?start=2023-07-01T01:30:00-0400&rule=2d&first=2020-07-31T02:41:00Z&last=2020-08-07T06:53:00Z&uri=${ENCODED_FEED_URL}`,
  );

  await page.getByText('▲').click();
  await page.getByText('▼').click();
  expect(await getRescheduledDates(page)).toEqual([
    'Jul 1st, 2023',
    'Jul 3rd, 2023',
    'Jul 5th, 2023',
  ]);
});

async function getRescheduledDates(page: Page, limit = 3) {
  const rescheduled = await page.$$('table td.rescheduled');
  return Promise.all(rescheduled.slice(0, limit).map((td) => td.textContent()));
}
