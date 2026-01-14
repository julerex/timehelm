#!/usr/bin/env node
/**
 * GLB/GLTF inspector - prints a human-readable summary of the contents.
 *
 * Usage:
 *   node client/scripts/inspect-glb.mjs client/public/assets/bedDouble.glb
 *
 * Or:
 *   npm run --prefix client inspect:glb -- client/public/assets/bedDouble.glb
 */
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

import { NodeIO } from '@gltf-transform/core';
import { KHRMaterialsUnlit } from '@gltf-transform/extensions';

function fmtVec3(v) {
  if (!v) return '(null)';
  const [x, y, z] = v;
  return `(${x.toFixed(3)}, ${y.toFixed(3)}, ${z.toFixed(3)})`;
}

function fmtQuat(q) {
  if (!q) return '(null)';
  const [x, y, z, w] = q;
  return `(${x.toFixed(3)}, ${y.toFixed(3)}, ${z.toFixed(3)}, ${w.toFixed(3)})`;
}

function indent(level) {
  return '  '.repeat(level);
}

function safeName(obj, fallback) {
  const n = obj?.getName?.();
  if (typeof n === 'string' && n.trim().length > 0) return n;
  return fallback;
}

function accessorSummary(accessor) {
  if (!accessor) return '(none)';
  const type = accessor.getType?.() ?? 'UNKNOWN';
  const count = accessor.getCount?.() ?? 0;
  const componentType = accessor.getComponentType?.() ?? 'UNKNOWN';
  const normalized = accessor.getNormalized?.() ? ' normalized' : '';
  return `${type} x ${count} (${componentType}${normalized})`;
}

function materialSummary(material) {
  if (!material) return '(none)';
  const name = safeName(material, '(unnamed material)');
  const alphaMode = material.getAlphaMode?.() ?? 'OPAQUE';
  const doubleSided = material.getDoubleSided?.() ? 'doubleSided' : '';
  const unlit = material.getUnlit?.() ? 'unlit' : '';

  const baseColor = material.getBaseColorFactor?.();
  const metalness = material.getMetallicFactor?.();
  const roughness = material.getRoughnessFactor?.();
  const emissive = material.getEmissiveFactor?.();

  const bcTex = material.getBaseColorTexture?.();
  const mrTex = material.getMetallicRoughnessTexture?.();
  const nTex = material.getNormalTexture?.();
  const oTex = material.getOcclusionTexture?.();
  const eTex = material.getEmissiveTexture?.();

  const texName = (t) => (t ? safeName(t, '(unnamed texture)') : null);

  const parts = [
    name,
    `alpha=${alphaMode}`,
    doubleSided,
    unlit,
    baseColor ? `baseColor=[${baseColor.map((c) => c.toFixed(3)).join(', ')}]` : null,
    typeof metalness === 'number' ? `metal=${metalness.toFixed(3)}` : null,
    typeof roughness === 'number' ? `rough=${roughness.toFixed(3)}` : null,
    emissive ? `emissive=[${emissive.map((c) => c.toFixed(3)).join(', ')}]` : null,
    texName(bcTex) ? `baseColorTex=${texName(bcTex)}` : null,
    texName(mrTex) ? `metalRoughTex=${texName(mrTex)}` : null,
    texName(nTex) ? `normalTex=${texName(nTex)}` : null,
    texName(oTex) ? `occlusionTex=${texName(oTex)}` : null,
    texName(eTex) ? `emissiveTex=${texName(eTex)}` : null,
  ].filter(Boolean);

  return parts.join(' | ');
}

function printNodeTree(node, level = 0) {
  const name = safeName(node, '(unnamed node)');
  const mesh = node.getMesh?.();
  const camera = node.getCamera?.();
  const skin = node.getSkin?.();

  const t = node.getTranslation?.();
  const r = node.getRotation?.();
  const s = node.getScale?.();

  const flags = [];
  if (mesh) flags.push(`mesh=${safeName(mesh, '(unnamed mesh)')}`);
  if (camera) flags.push(`camera=${safeName(camera, '(unnamed camera)')}`);
  if (skin) flags.push(`skin=${safeName(skin, '(unnamed skin)')}`);

  console.log(
    `${indent(level)}- ${name}` +
      (flags.length ? ` (${flags.join(', ')})` : '') +
      ` t=${fmtVec3(t)} r=${fmtQuat(r)} s=${fmtVec3(s)}`
  );

  const children = node.listChildren?.() ?? [];
  for (const child of children) {
    printNodeTree(child, level + 1);
  }
}

async function main() {
  const input = process.argv[2];
  if (!input) {
    console.error('Usage: node inspect-glb.mjs <path-to-.glb-or-.gltf>');
    process.exit(1);
  }

  const inputPath = path.resolve(process.cwd(), input);
  if (!fs.existsSync(inputPath)) {
    console.error(`File not found: ${inputPath}`);
    process.exit(1);
  }

  const io = new NodeIO();
  io.registerExtensions([KHRMaterialsUnlit]);
  const doc = await io.read(inputPath);
  const root = doc.getRoot();

  console.log(`File: ${inputPath}`);
  console.log('');

  // --- High-level counts ---
  const scenes = root.listScenes();
  const nodes = root.listNodes();
  const meshes = root.listMeshes();
  const materials = root.listMaterials();
  const textures = root.listTextures();
  const accessors = root.listAccessors();
  const animations = root.listAnimations();
  const skins = root.listSkins();
  const cameras = root.listCameras();

  const imageBytesTotal = textures.reduce((sum, tex) => {
    const img = tex.getImage?.();
    return sum + (img?.byteLength ?? 0);
  }, 0);

  console.log('== Summary ==');
  console.log(`Scenes:     ${scenes.length}`);
  console.log(`Nodes:      ${nodes.length}`);
  console.log(`Meshes:     ${meshes.length}`);
  console.log(`Materials:  ${materials.length}`);
  console.log(`Textures:   ${textures.length}`);
  console.log(`Images:     ${textures.filter((t) => (t.getImage?.()?.byteLength ?? 0) > 0).length} (${imageBytesTotal} bytes total)`);
  console.log(`Accessors:  ${accessors.length}`);
  console.log(`Animations: ${animations.length}`);
  console.log(`Skins:      ${skins.length}`);
  console.log(`Cameras:    ${cameras.length}`);
  console.log('');

  // --- Scenes / node tree ---
  console.log('== Scene Graph ==');
  if (scenes.length === 0) {
    console.log('(no scenes)');
  }
  for (const [i, scene] of scenes.entries()) {
    console.log(`Scene[${i}]: ${safeName(scene, '(unnamed scene)')}`);
    const roots = scene.listChildren?.() ?? [];
    for (const node of roots) {
      printNodeTree(node, 1);
    }
  }
  console.log('');

  // --- Meshes / primitives ---
  console.log('== Meshes ==');
  if (meshes.length === 0) {
    console.log('(no meshes)');
  }
  for (const [i, mesh] of meshes.entries()) {
    console.log(`Mesh[${i}]: ${safeName(mesh, '(unnamed mesh)')}`);
    const prims = mesh.listPrimitives?.() ?? [];
    console.log(`  Primitives: ${prims.length}`);
    for (const [p, prim] of prims.entries()) {
      const mode = prim.getMode?.() ?? 'TRIANGLES';
      const mat = prim.getMaterial?.();
      const pos = prim.getAttribute?.('POSITION');
      const nor = prim.getAttribute?.('NORMAL');
      const uv0 = prim.getAttribute?.('TEXCOORD_0');
      const col0 = prim.getAttribute?.('COLOR_0');
      const joints0 = prim.getAttribute?.('JOINTS_0');
      const weights0 = prim.getAttribute?.('WEIGHTS_0');
      const indices = prim.getIndices?.();

      console.log(`  - Primitive[${p}] mode=${mode}`);
      console.log(`    Material: ${materialSummary(mat)}`);
      console.log(`    POSITION: ${accessorSummary(pos)}`);
      if (nor) console.log(`    NORMAL:   ${accessorSummary(nor)}`);
      if (uv0) console.log(`    UV0:      ${accessorSummary(uv0)}`);
      if (col0) console.log(`    COLOR0:   ${accessorSummary(col0)}`);
      if (joints0) console.log(`    JOINTS0:  ${accessorSummary(joints0)}`);
      if (weights0) console.log(`    WEIGHTS0: ${accessorSummary(weights0)}`);
      if (indices) console.log(`    Indices:  ${accessorSummary(indices)}`);
    }
  }
  console.log('');

  // --- Materials ---
  console.log('== Materials ==');
  if (materials.length === 0) {
    console.log('(no materials)');
  }
  for (const [i, mat] of materials.entries()) {
    console.log(`Material[${i}]: ${materialSummary(mat)}`);
  }
  console.log('');

  // --- Textures / images ---
  console.log('== Textures ==');
  if (textures.length === 0) {
    console.log('(no textures)');
  }
  for (const [i, tex] of textures.entries()) {
    const imgBytes = tex.getImage?.();
    const sampler = tex.getSampler?.();
    console.log(`Texture[${i}]: ${safeName(tex, '(unnamed texture)')}`);
    console.log(`  Image: mime=${tex.getMimeType?.() ?? '(unknown)'} bytes=${imgBytes?.byteLength ?? 0} uri=${tex.getURI?.() ?? ''}`.trim());
    if (sampler) {
      console.log(
        `  Sampler: wrapS=${sampler.getWrapS?.() ?? '(unknown)'} wrapT=${sampler.getWrapT?.() ?? '(unknown)'} min=${sampler.getMinFilter?.() ?? '(unknown)'} mag=${sampler.getMagFilter?.() ?? '(unknown)'}`
      );
    }
  }
  console.log('');

  // --- Animations ---
  console.log('== Animations ==');
  if (animations.length === 0) {
    console.log('(no animations)');
  }
  for (const [i, anim] of animations.entries()) {
    console.log(`Animation[${i}]: ${safeName(anim, '(unnamed animation)')}`);
    const channels = anim.listChannels?.() ?? [];
    console.log(`  Channels: ${channels.length}`);
    for (const [c, ch] of channels.entries()) {
      const targetNode = ch.getTargetNode?.();
      const pathName = ch.getTargetPath?.() ?? '(unknown)';
      const sampler = ch.getSampler?.();
      const inputAcc = sampler?.getInput?.();
      const outputAcc = sampler?.getOutput?.();
      console.log(`  - Channel[${c}]: node=${safeName(targetNode, '(unnamed node)')} path=${pathName}`);
      console.log(`    input:  ${accessorSummary(inputAcc)}`);
      console.log(`    output: ${accessorSummary(outputAcc)}`);
    }
  }
}

main().catch((err) => {
  console.error(err?.stack ?? String(err));
  process.exit(1);
});


