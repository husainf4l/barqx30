import { Component, OnInit, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { AuthService, User } from '../../services/auth.service';
import { StorageService, S3Object, ListBucketResponse, Bucket } from '../../services/storage.service';

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="dashboard">
      <header>
        <div class="header-content">
          <h1>⚡ BARQ X30 Dashboard</h1>
          <div class="user-info">
            <span class="username">{{ user()?.name }}</span>
            <span class="role-badge" [class.super-admin]="user()?.role === 'super_admin'">
              {{ user()?.role }}
            </span>
            <button class="logout-btn" (click)="logout()">Logout</button>
          </div>
        </div>
      </header>

      <div class="container">
        <!-- Storage Overview -->
        <div class="stats-grid">
          <div class="stat-card">
            <div class="stat-icon">📦</div>
            <div class="stat-info">
              <div class="stat-label">Storage Used</div>
              <div class="stat-value">{{ formatBytes(user()?.storage_used || 0) }}</div>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-icon">💾</div>
            <div class="stat-info">
              <div class="stat-label">Storage Quota</div>
              <div class="stat-value">{{ formatBytes(user()?.storage_quota || 0) }}</div>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-icon">📊</div>
            <div class="stat-info">
              <div class="stat-label">Usage</div>
              <div class="stat-value">{{ usagePercentage }}%</div>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-icon">⚡</div>
            <div class="stat-info">
              <div class="stat-label">Objects</div>
              <div class="stat-value">{{ objects().length }}</div>
            </div>
          </div>
        </div>

        <!-- Bucket Management -->
        <div class="bucket-management">
          <h2>📂 My Buckets</h2>
          
          <div class="create-bucket">
            <input 
              type="text" 
              [(ngModel)]="newBucketName" 
              placeholder="Enter bucket name (lowercase, no spaces)"
              (keyup.enter)="createBucket()"
            />
            <button (click)="createBucket()" [disabled]="!newBucketName() || loading()">
              ➕ Create Bucket
            </button>
          </div>

          <div class="error" *ngIf="error()">{{ error() }}</div>
          <div class="success" *ngIf="success()">{{ success() }}</div>

          <div class="loading" *ngIf="loading() && buckets().length === 0">Loading buckets...</div>

          <div class="buckets-grid" *ngIf="buckets().length > 0">
            <div 
              *ngFor="let bucket of buckets()" 
              class="bucket-card"
              [class.selected]="currentBucket() === bucket.name"
              (click)="selectBucket(bucket.name)"
            >
              <div class="bucket-icon">🗄️</div>
              <div class="bucket-name">{{ bucket.name }}</div>
              <div class="bucket-date">Created: {{ formatDate(bucket.created_at) }}</div>
            </div>
          </div>

          <div class="empty-buckets" *ngIf="!loading() && buckets().length === 0">
            <p>📦 No buckets yet. Create one above to get started!</p>
          </div>
        </div>

        <!-- File Manager -->
        <div class="file-manager" *ngIf="currentBucket()">
          <h2>📁 Files in "{{ currentBucket() }}"</h2>
          <div class="toolbar">
            <div class="bucket-selector">
              <label>Bucket:</label>
              <input 
                type="text" 
                [(ngModel)]="currentBucket" 
                placeholder="Enter bucket name"
                (keyup.enter)="loadBucket()"
              />
              <button (click)="loadBucket()" [disabled]="!currentBucket()">Load</button>
            </div>
            
            <div class="upload-section">
              <input 
                type="file" 
                #fileInput 
                (change)="onFileSelected($event)"
                style="display: none"
              />
              <button (click)="fileInput.click()" [disabled]="!currentBucket()">
                📤 Upload File
              </button>
            </div>
          </div>

          <div class="error" *ngIf="error()">{{ error() }}</div>
          <div class="success" *ngIf="success()">{{ success() }}</div>

          <div class="files-list" *ngIf="currentBucket()">
            <h3>Files in "{{ currentBucket() }}"</h3>
            
            <div class="loading" *ngIf="loading()">Loading...</div>

            <table *ngIf="!loading() && objects().length > 0">
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Size</th>
                  <th>ETag</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                <tr *ngFor="let obj of objects()">
                  <td>{{ obj.key }}</td>
                  <td>{{ formatBytes(obj.size) }}</td>
                  <td class="etag">{{ obj.etag }}</td>
                  <td>
                    <button class="btn-download" (click)="downloadFile(obj.key)">
                      ⬇️ Download
                    </button>
                    <button class="btn-delete" (click)="deleteFile(obj.key)">
                      🗑️ Delete
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>

            <div class="empty-state" *ngIf="!loading() && objects().length === 0">
              <p>No files in this bucket</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  `,
  styles: [`
    .dashboard {
      min-height: 100vh;
      background: #f5f7fa;
    }

    header {
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      color: white;
      padding: 20px 0;
      box-shadow: 0 2px 10px rgba(0,0,0,0.1);
    }

    .header-content {
      max-width: 1200px;
      margin: 0 auto;
      padding: 0 20px;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    h1 {
      margin: 0;
      font-size: 28px;
    }

    .user-info {
      display: flex;
      align-items: center;
      gap: 15px;
    }

    .username {
      font-weight: 500;
    }

    .role-badge {
      background: rgba(255,255,255,0.2);
      padding: 5px 12px;
      border-radius: 15px;
      font-size: 12px;
      text-transform: uppercase;
    }

    .role-badge.super-admin {
      background: #f39c12;
      font-weight: bold;
    }

    .logout-btn {
      background: rgba(255,255,255,0.2);
      color: white;
      border: none;
      padding: 8px 20px;
      border-radius: 6px;
      cursor: pointer;
      font-weight: 500;
    }

    .logout-btn:hover {
      background: rgba(255,255,255,0.3);
    }

    .container {
      max-width: 1200px;
      margin: 0 auto;
      padding: 30px 20px;
    }

    .stats-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
      gap: 20px;
      margin-bottom: 30px;
    }

    .stat-card {
      background: white;
      padding: 20px;
      border-radius: 12px;
      box-shadow: 0 2px 8px rgba(0,0,0,0.1);
      display: flex;
      align-items: center;
      gap: 15px;
    }

    .stat-icon {
      font-size: 36px;
    }

    .stat-label {
      color: #666;
      font-size: 14px;
      margin-bottom: 5px;
    }

    .stat-value {
      font-size: 24px;
      font-weight: bold;
      color: #333;
    }
    .bucket-management {
      background: white;
      padding: 25px;
      border-radius: 12px;
      box-shadow: 0 2px 8px rgba(0,0,0,0.1);
      margin-bottom: 30px;
    }

    .bucket-management h2 {
      margin: 0 0 20px 0;
      color: #333;
      font-size: 22px;
    }

    .create-bucket {
      display: flex;
      gap: 10px;
      margin-bottom: 20px;
    }

    .create-bucket input {
      flex: 1;
      padding: 10px 15px;
      border: 1px solid #ddd;
      border-radius: 6px;
      font-size: 14px;
    }

    .create-bucket button {
      padding: 10px 20px;
      white-space: nowrap;
    }

    .buckets-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
      gap: 15px;
      margin-top: 20px;
    }

    .bucket-card {
      background: #f8f9fa;
      padding: 20px;
      border-radius: 8px;
      border: 2px solid #e0e0e0;
      cursor: pointer;
      transition: all 0.2s;
      text-align: center;
    }

    .bucket-card:hover {
      border-color: #667eea;
      transform: translateY(-2px);
      box-shadow: 0 4px 12px rgba(102, 126, 234, 0.2);
    }

    .bucket-card.selected {
      border-color: #667eea;
      background: #f0f3ff;
    }

    .bucket-icon {
      font-size: 40px;
      margin-bottom: 10px;
    }

    .bucket-name {
      font-weight: 600;
      color: #333;
      margin-bottom: 5px;
    }

    .bucket-date {
      font-size: 12px;
      color: #999;
    }

    .empty-buckets {
      text-align: center;
      padding: 40px;
      color: #999;
      font-size: 16px;
    }
    .file-manager {
      background: white;
      padding: 25px;
      border-radius: 12px;
      box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    }

    .toolbar {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 20px;
      flex-wrap: wrap;
      gap: 15px;
    }

    .bucket-selector {
      display: flex;
      align-items: center;
      gap: 10px;
    }

    .bucket-selector label {
      font-weight: 500;
    }

    input[type="text"] {
      padding: 8px 12px;
      border: 1px solid #ddd;
      border-radius: 6px;
      font-size: 14px;
    }

    button {
      padding: 8px 16px;
      background: #667eea;
      color: white;
      border: none;
      border-radius: 6px;
      cursor: pointer;
      font-weight: 500;
    }

    button:hover:not(:disabled) {
      background: #5568d3;
    }

    button:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }

    .error {
      background: #fee;
      color: #c33;
      padding: 10px 15px;
      border-radius: 6px;
      margin-bottom: 15px;
    }

    .success {
      background: #efe;
      color: #3c3;
      padding: 10px 15px;
      border-radius: 6px;
      margin-bottom: 15px;
    }

    .files-list h3 {
      margin: 0 0 15px 0;
      color: #333;
    }

    .loading {
      text-align: center;
      padding: 40px;
      color: #666;
    }

    table {
      width: 100%;
      border-collapse: collapse;
    }

    th {
      background: #f8f9fa;
      padding: 12px;
      text-align: left;
      font-weight: 600;
      color: #333;
      border-bottom: 2px solid #dee2e6;
    }

    td {
      padding: 12px;
      border-bottom: 1px solid #dee2e6;
    }

    .etag {
      font-family: monospace;
      font-size: 12px;
      color: #666;
    }

    .btn-download, .btn-delete {
      padding: 6px 12px;
      font-size: 12px;
      margin-right: 5px;
    }

    .btn-delete {
      background: #e74c3c;
    }

    .btn-delete:hover:not(:disabled) {
      background: #c0392b;
    }

    .empty-state {
      text-align: center;
      padding: 40px;
      color: #999;
    }
  `]
})
export class DashboardComponent implements OnInit {
  user = signal<User | null>(null);
  currentBucket = signal('');
  buckets = signal<Bucket[]>([]);
  newBucketName = signal('');
  objects = signal<S3Object[]>([]);
  loading = signal(false);
  error = signal('');
  success = signal('');

  constructor(
    private authService: AuthService,
    private storageService: StorageService,
    private router: Router
  ) {}

  ngOnInit(): void {
    // Subscribe to user changes
    this.authService.currentUser$.subscribe(user => {
      this.user.set(user);
    });

    // Check if user is authenticated
    if (!this.authService.isAuthenticated()) {
      this.router.navigate(['/login']);
      return;
    }

    // User is in localStorage, load buckets
    if (this.user()) {
      this.loadBuckets();
    }

    // Refresh user data from backend in the background
    this.authService.getCurrentUser().subscribe({
      next: () => {
        // User data refreshed successfully, reload buckets if needed
        if (!this.buckets().length) {
          this.loadBuckets();
        }
      },
      error: (err: any) => {
        console.error('Failed to refresh user data:', err);
        // Only logout if it's an auth error (401)
        if (err.status === 401) {
          this.authService.logout();
          this.router.navigate(['/login']);
        }
      }
    });
  }

  get usagePercentage(): number {
    const user = this.user();
    if (!user || user.storage_quota === 0) return 0;
    return Math.round((user.storage_used / user.storage_quota) * 100);
  }

  formatBytes(bytes: number): string {
    return this.storageService.formatBytes(bytes);
  }

  formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleDateString();
  }

  loadBuckets(): void {
    this.loading.set(true);
    this.error.set('');
    
    console.log('Loading buckets...');
    this.storageService.listBuckets().subscribe({
      next: (response) => {
        console.log('Buckets loaded:', response);
        this.buckets.set(response.buckets);
        this.loading.set(false);
        console.log('Buckets array:', this.buckets(), 'Loading:', this.loading());
      },
      error: (err: any) => {
        this.error.set('Failed to load buckets');
        this.loading.set(false);
        console.error('Load buckets error:', err);
      }
    });
  }

  createBucket(): void {
    if (!this.newBucketName()) return;
    
    this.loading.set(true);
    this.error.set('');
    this.success.set('');

    this.storageService.createBucket(this.newBucketName()).subscribe({
      next: (bucket) => {
        this.success.set(`Bucket "${bucket.name}" created successfully!`);
        this.newBucketName.set('');
        this.loadBuckets();
        this.loading.set(false);
      },
      error: (err: any) => {
        this.error.set('Failed to create bucket');
        this.loading.set(false);
        console.error('Create bucket error:', err);
      }
    });
  }

  selectBucket(bucketName: string): void {
    this.currentBucket.set(bucketName);
    this.loadBucket();
  }

  loadBucket(): void {
    if (!this.currentBucket()) return;
    
    this.loading.set(true);
    this.error.set('');
    this.success.set('');

    this.storageService.listBucketObjects(this.currentBucket()).subscribe({
      next: (response: ListBucketResponse) => {
        this.objects.set(response.objects || []);
        this.loading.set(false);
      },
      error: (err: any) => {
        this.error.set('Failed to load bucket. Make sure it exists.');
        this.loading.set(false);
        console.error('Load bucket error:', err);
      }
    });
  }

  onFileSelected(event: any): void {
    const file: File = event.target.files[0];
    if (!file || !this.currentBucket()) return;

    this.loading.set(true);
    this.error.set('');
    this.success.set('');

    this.storageService.uploadFile(this.currentBucket(), file.name, file).subscribe({
      next: () => {
        this.success.set(`File "${file.name}" uploaded successfully!`);
        this.loading.set(false);
        this.loadBucket(); // Reload bucket to show new file
      },
      error: (err: any) => {
        this.error.set('Failed to upload file');
        this.loading.set(false);
        console.error('Upload error:', err);
      }
    });
  }

  downloadFile(key: string): void {
    this.storageService.downloadFile(this.currentBucket(), key).subscribe({
      next: (blob) => {
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = key;
        a.click();
        window.URL.revokeObjectURL(url);
      },
      error: (err: any) => {
        this.error.set('Failed to download file');
        console.error('Download error:', err);
      }
    });
  }

  deleteFile(key: string): void {
    if (!confirm(`Delete "${key}"?`)) return;

    this.storageService.deleteFile(this.currentBucket(), key).subscribe({
      next: () => {
        this.success.set(`File "${key}" deleted successfully!`);
        this.loadBucket(); // Reload bucket
      },
      error: (err: any) => {
        this.error.set('Failed to delete file');
        console.error('Delete error:', err);
      }
    });
  }

  logout(): void {
    this.authService.logout();
    this.router.navigate(['/login']);
  }
}
