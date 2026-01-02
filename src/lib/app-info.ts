import pkg from "../../package.json";

interface PackageJson {
  name: string;
  version: string;
  description: string;
  author: {
    name: string;
    email: string;
    url: string;
  };
  repository: string;
}

export const appInfo = {
  name: pkg.name,
  version: pkg.version,
  description: pkg.description,
  author: {
    name: pkg.author.name,
    email: pkg.author.email,
    url: pkg.author.url,
  },
  repository: pkg.repository,
} satisfies PackageJson;
