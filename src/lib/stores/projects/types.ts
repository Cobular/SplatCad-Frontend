export interface CloudProjectData {
  id: number;
  name: string;
  description?: string;
  createdAt: string;
  cloudFiles: FileMapping;
}

export interface CloudProjectMapping {
  [projectId: string]: CloudProjectData;
}

export interface File {
  path: string;
  name: string;
  updateDate: Date;
  hash: string;
}

export interface FileMapping {
  [filePath: string]: File;
}

export interface ProjectFileMapping {
  [projectId: string]: FileMapping;
}

export interface WholeProject {
  metadata: CloudProjectData;
  localFiles: FileMapping;
}