import { test, expect } from '@playwright/test';

// a basic test that runs a file
// checks the output
// and checks that decompiled/data tabs are correct
test('basic_running', async ({ page }) => {
  await page.goto('http://localhost:8080/');
  await page.getByText('Load', { exact: true }).click();
  await page.getByText('Load', { exact: true }).setInputFiles('../../../test_files/success/print10.s');
  // file loaded correctly
  // check editor has a main and a loop label
  page.getByText('main:', { exact: true });
  page.getByText('loop:', { exact: true });

  // click the save button
  await page.getByRole('button', { name: 'Save' }).click();
  // wait for run button to be enabled
  await page.waitForSelector('#run_button:not([disabled])');

  // check decompiled tab contains 28 rows of instructions
  await page.getByRole('button', { name: 'decompiled' }).click();
  page.getByText('0x00400004 [0x34010005]    ori    $at, $zero, 5             ; [8] bge  $s0, 5, end     # if (i >= 5) goto end;', { exact: true });
  
  // check data tab is empty
  await page.getByRole('button', { name: 'data' }).click();
  const data_output_div = page.locator('#data_output');
  expect(await data_output_div.innerText()).toStrictEqual("");

  // run the program
  await page.getByRole('button', { name: 'Run' }).click();
  const program_io_output = page.locator('#program_io_output').nth(0);

  program_io_output.getByText('3\n9\n27\n81\n243', { exact: true });

});
