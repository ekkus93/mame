import type { PlatformPath } from "../backend/pathConfiguration";

export type ContentPathKey = "romPaths" | "softwarePaths" | "chdPaths";

export const CONTENT_PATH_GROUPS: ReadonlyArray<{
  key: ContentPathKey;
  label: string;
  addLabel: string;
}> = [
  { key: "romPaths", label: "ROM paths", addLabel: "Add ROM directory" },
  { key: "softwarePaths", label: "Software paths", addLabel: "Add software directory" },
  { key: "chdPaths", label: "CHD paths", addLabel: "Add CHD directory" },
];

export function platformPathKey(path: PlatformPath): string {
  return typeof path === "string" ? `utf8:${path}` : `${path.encoding}:${path.data}`;
}

export function displayPlatformPath(path: PlatformPath): string {
  if (typeof path === "string") {
    return path;
  }
  return `Encoded platform path (${path.encoding}): ${path.data}`;
}

export function addUniquePath(paths: PlatformPath[], path: PlatformPath): PlatformPath[] {
  const key = platformPathKey(path);
  if (paths.some((candidate) => platformPathKey(candidate) === key)) {
    return paths;
  }
  return [...paths, path];
}

export function removePath(paths: PlatformPath[], index: number): PlatformPath[] {
  return paths.filter((_, candidateIndex) => candidateIndex !== index);
}

export function movePath(paths: PlatformPath[], index: number, direction: -1 | 1): PlatformPath[] {
  const target = index + direction;
  if (index < 0 || index >= paths.length || target < 0 || target >= paths.length) {
    return paths;
  }

  const current = paths[index];
  const replacement = paths[target];
  if (current === undefined || replacement === undefined) {
    return paths;
  }

  const next = [...paths];
  next[index] = replacement;
  next[target] = current;
  return next;
}
