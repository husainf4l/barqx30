import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders, HttpRequest, HttpEvent } from '@angular/common/http';
import { Observable } from 'rxjs';
import { AuthService } from './auth.service';

export interface S3Object {
  key: string;
  size: number;
  etag: string;
  content_type: string;
  created_at: string;
}

export interface ListBucketResponse {
  bucket: string;
  objects: S3Object[];
  count: number;
}

export interface Bucket {
  id: number;
  name: string;
  user_id: number;
  created_at: string;
}

export interface BucketListResponse {
  buckets: Bucket[];
  count: number;
}

@Injectable({
  providedIn: 'root'
})
export class StorageService {
  private apiUrl = '/api';
  private s3Url = '/s3';

  constructor(
    private http: HttpClient,
    private authService: AuthService
  ) {}

  private getHeaders(): HttpHeaders {
    const token = this.authService.getToken();
    return new HttpHeaders({
      'Authorization': `Bearer ${token}`
    });
  }

  // REST API - Bucket management
  createBucket(name: string): Observable<Bucket> {
    return this.http.post<Bucket>(
      `${this.apiUrl}/buckets`,
      { name },
      { headers: this.getHeaders() }
    );
  }

  listBuckets(): Observable<BucketListResponse> {
    return this.http.get<BucketListResponse>(
      `${this.apiUrl}/buckets`,
      { headers: this.getHeaders() }
    );
  }

  getBucket(name: string): Observable<Bucket> {
    return this.http.get<Bucket>(
      `${this.apiUrl}/buckets/${name}`,
      { headers: this.getHeaders() }
    );
  }

  listBucketObjects(bucketName: string): Observable<ListBucketResponse> {
    return this.http.get<ListBucketResponse>(
      `${this.apiUrl}/buckets/${bucketName}/objects`,
      { headers: this.getHeaders() }
    );
  }

  // S3-compatible API - Object operations
  uploadFile(bucketName: string, key: string, file: File): Observable<HttpEvent<any>> {
    const headers = this.getHeaders().set('Content-Type', file.type || 'application/octet-stream');
    const req = new HttpRequest('PUT', `${this.s3Url}/${bucketName}/${key}`, file, {
      headers,
      reportProgress: true,
      responseType: 'text',
    });
    return this.http.request(req);
  }

  downloadFile(bucketName: string, key: string): Observable<Blob> {
    return this.http.get(
      `${this.s3Url}/${bucketName}/${key}`,
      { headers: this.getHeaders(), responseType: 'blob' }
    );
  }

  deleteFile(bucketName: string, key: string): Observable<any> {
    return this.http.delete(
      `${this.s3Url}/${bucketName}/${key}`,
      { headers: this.getHeaders(), responseType: 'text' }
    );
  }

  formatBytes(bytes: number): string {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  }
}
