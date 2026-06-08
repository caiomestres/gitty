#!/usr/bin/env node
/**
 * Icon asset generator for Gitty.
 *
 * Uses two rendering tiers from the same character:
 *   --detailed : full-body mascot for 128px+ sizes
 *   --simplified : reduced-detail circle icon for 16-64px sizes
 *
 * Usage:
 *   node scripts/generate-icons.mjs --detailed path/to/full.png --simplified path/to/icon.png [--svg path/to/icon.svg]
 *
 * Prerequisites:
 *   npm install --save-dev sharp png-to-ico
 */

import { existsSync, mkdirSync, copyFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import sharp from 'sharp';
import pngToIco from 'png-to-ico';
import { writeFile } from 'node:fs/promises';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '..');
const ICONS_DIR = resolve(ROOT, 'src-tauri/icons');
const TRAY_DIR = resolve(ICONS_DIR, 'tray');
const DOCS_DIR = resolve(ROOT, 'docs/assets');

const SMALL_SIZES = [16, 22, 24, 32, 64];
const LARGE_SIZES = [128, 256, 512, 1024];
const ICO_SIZES = [16, 32, 48, 256];
const TRAY_SIZES = [22, 24];
const OG_WIDTH = 1200;
const OG_HEIGHT = 630;
const TIER_THRESHOLD = 128;

function ensureDir(dir) {
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
}

function sourceForSize(size, detailed, simplified) {
  return size >= TIER_THRESHOLD ? detailed : simplified;
}

async function generatePngs(detailed, simplified) {
  console.log('Generating PNG icons...');
  const allSizes = [...SMALL_SIZES, ...LARGE_SIZES].sort((a, b) => a - b);
  for (const size of allSizes) {
    const src = sourceForSize(size, detailed, simplified);
    const tier = size >= TIER_THRESHOLD ? 'detailed' : 'simplified';
    const output = resolve(ICONS_DIR, `${size}x${size}.png`);
    await sharp(src).resize(size, size, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } }).png().toFile(output);
    console.log(`  ✓ ${size}x${size}.png (${tier})`);
  }

  const retina = resolve(ICONS_DIR, '128x128@2x.png');
  await sharp(detailed).resize(256, 256, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } }).png().toFile(retina);
  console.log('  ✓ 128x128@2x.png (256px actual, detailed)');
}

async function generateTrayIcons(simplified) {
  console.log('Generating tray icons...');
  for (const size of TRAY_SIZES) {
    const output = resolve(TRAY_DIR, `tray-${size}x${size}.png`);
    await sharp(simplified).resize(size, size, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } }).png().toFile(output);
    console.log(`  ✓ tray/tray-${size}x${size}.png`);
  }
}

async function generateIco(detailed, simplified) {
  console.log('Generating ICO (multi-resolution)...');
  const buffers = await Promise.all(
    ICO_SIZES.map((size) => {
      const src = sourceForSize(size, detailed, simplified);
      return sharp(src).resize(size, size, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } }).png().toBuffer();
    })
  );
  const ico = await pngToIco(buffers);
  const output = resolve(ICONS_DIR, 'icon.ico');
  await writeFile(output, ico);
  console.log('  ✓ icon.ico');

  const faviconOutput = resolve(DOCS_DIR, 'favicon.ico');
  await writeFile(faviconOutput, ico);
  console.log('  ✓ docs/assets/favicon.ico');
}

async function generateOgImage(detailed) {
  console.log('Generating OG image (1200x630)...');
  const output = resolve(DOCS_DIR, 'og-image.png');
  await sharp(detailed)
    .resize(OG_WIDTH, OG_HEIGHT, { fit: 'contain', background: { r: 34, g: 34, b: 42, alpha: 1 } })
    .png()
    .toFile(output);
  console.log('  ✓ docs/assets/og-image.png');
}

function copySvg(svgPath) {
  console.log('Copying SVG source...');
  const iconsSvg = resolve(ICONS_DIR, 'icon.svg');
  const docsSvg = resolve(DOCS_DIR, 'favicon.svg');

  copyFileSync(svgPath, iconsSvg);
  console.log(`  ✓ src-tauri/icons/icon.svg`);

  copyFileSync(svgPath, docsSvg);
  console.log(`  ✓ docs/assets/favicon.svg`);
}

async function generateIcns(detailed, simplified) {
  console.log('ICNS generation...');
  if (process.platform === 'darwin') {
    const { execSync } = await import('node:child_process');
    const iconsetDir = resolve(ICONS_DIR, 'icon.iconset');
    ensureDir(iconsetDir);

    const macSizes = [16, 32, 64, 128, 256, 512];
    for (const size of macSizes) {
      copyFileSync(resolve(ICONS_DIR, `${size}x${size}.png`), resolve(iconsetDir, `icon_${size}x${size}.png`));
      const double = size * 2;
      if (existsSync(resolve(ICONS_DIR, `${double}x${double}.png`))) {
        copyFileSync(resolve(ICONS_DIR, `${double}x${double}.png`), resolve(iconsetDir, `icon_${size}x${size}@2x.png`));
      }
    }

    execSync(`iconutil -c icns "${iconsetDir}" -o "${resolve(ICONS_DIR, 'icon.icns')}"`);
    execSync(`rm -rf "${iconsetDir}"`);
    console.log('  ✓ icon.icns (via iconutil)');
  } else {
    console.log('  ⚠ Skipped — iconutil only available on macOS.');
    console.log('    To generate icon.icns:');
    console.log('    1. Run this script on macOS, OR');
    console.log('    2. Use an online PNG-to-ICNS converter with the 1024x1024.png');
  }
}

async function main() {
  const { values } = parseArgs({
    options: {
      detailed: { type: 'string', short: 'd' },
      simplified: { type: 'string', short: 's' },
      svg: { type: 'string' },
    },
  });

  if (!values.detailed || !values.simplified) {
    console.error('Usage: node scripts/generate-icons.mjs --detailed <full-body.png> --simplified <icon.png> [--svg <icon.svg>]');
    console.error('');
    console.error('  --detailed    Full-body mascot artwork (used for 128px+ sizes)');
    console.error('  --simplified  Reduced-detail icon (used for 16-64px sizes, tray, favicon)');
    console.error('  --svg         Optional SVG source to copy into icon locations');
    process.exit(1);
  }

  const detailed = resolve(values.detailed);
  const simplified = resolve(values.simplified);

  if (!existsSync(detailed)) { console.error(`Detailed source not found: ${detailed}`); process.exit(1); }
  if (!existsSync(simplified)) { console.error(`Simplified source not found: ${simplified}`); process.exit(1); }

  ensureDir(ICONS_DIR);
  ensureDir(TRAY_DIR);
  ensureDir(DOCS_DIR);

  console.log(`\nDetailed (128px+): ${detailed}`);
  console.log(`Simplified (<128px): ${simplified}\n`);

  await generatePngs(detailed, simplified);
  await generateTrayIcons(simplified);
  await generateIco(detailed, simplified);
  await generateOgImage(detailed);
  await generateIcns(detailed, simplified);

  if (values.svg) {
    const svgPath = resolve(values.svg);
    if (!existsSync(svgPath)) { console.error(`SVG file not found: ${svgPath}`); process.exit(1); }
    copySvg(svgPath);
  } else {
    console.log('\nNo --svg provided. Skipping SVG copy.');
  }

  console.log('\n✅ Icon generation complete!');
  console.log('\nRemaining manual steps:');
  if (process.platform !== 'darwin') {
    console.log('  - Generate icon.icns on macOS or via online converter');
  }
  console.log('  - Review all icons for quality at small sizes');
  console.log('  - Commit the generated assets');
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
