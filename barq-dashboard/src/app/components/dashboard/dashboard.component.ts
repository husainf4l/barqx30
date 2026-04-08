import { ChangeDetectionStrategy, Component, OnInit, computed, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { AuthService, User } from '../../services/auth.service';
import { StorageService, S3Object, ListBucketResponse, Bucket } from '../../services/storage.service';

@Component({
  selector: 'app-dashboard',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [FormsModule],
  template: `
    <div class="min-h-[100dvh] bg-background">

      <!-- ── Header ─────────────────────────────────────────────── -->
      <header class="bg-card border-b border-border sticky top-0 z-10">
        <div class="max-w-7xl mx-auto px-8 py-4 flex items-center justify-between">
          <p class="text-lg font-semibold tracking-tight text-foreground">BARQ X30</p>

          <div class="flex items-center gap-3">
            <span class="text-sm text-foreground/60">{{ user()?.name }}</span>

            <!-- Role badge -->
            @if (user()?.role === 'super_admin') {
              <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-semibold bg-amber-500/10 text-amber-600 ring-1 ring-inset ring-amber-500/20">
                super admin
              </span>
            } @else {
              <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-semibold bg-primary/10 text-primary ring-1 ring-inset ring-primary/20">
                {{ user()?.role }}
              </span>
            }

            <button
              (click)="logout()"
              class="rounded-md px-3 py-1.5 text-sm font-medium text-foreground hover:bg-muted transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 active:scale-[0.98]"
            >
              Sign out
            </button>
          </div>
        </div>
      </header>

      <!-- ── Page body ───────────────────────────────────────────── -->
      <main class="max-w-7xl mx-auto px-8 py-8 space-y-8">

        <!-- KPI stat cards -->
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">

          <div class="bg-card rounded-2xl border border-border shadow-sm p-6 animate-stagger" style="--index:0">
            <p class="text-xs font-bold uppercase tracking-widest text-muted-foreground">Storage used</p>
            <p class="text-2xl font-bold tracking-tight font-mono tabular-nums mt-2 text-foreground">
              {{ formatBytes(user()?.storage_used || 0) }}
            </p>
          </div>

          <div class="bg-card rounded-2xl border border-border shadow-sm p-6 animate-stagger" style="--index:1">
            <p class="text-xs font-bold uppercase tracking-widest text-muted-foreground">Storage quota</p>
            <p class="text-2xl font-bold tracking-tight font-mono tabular-nums mt-2 text-foreground">
              {{ formatBytes(user()?.storage_quota || 0) }}
            </p>
          </div>

          <div class="bg-card rounded-2xl border border-border shadow-sm p-6 animate-stagger" style="--index:2">
            <p class="text-xs font-bold uppercase tracking-widest text-muted-foreground">Usage</p>
            <p class="text-2xl font-bold tracking-tight font-mono tabular-nums mt-2 text-foreground">
              {{ usagePercentage() }}<span class="text-base text-muted-foreground">%</span>
            </p>
            <!-- Usage bar -->
            <div class="mt-3 h-1.5 bg-muted rounded-full overflow-hidden">
              <div
                class="h-full bg-primary rounded-full transition-all duration-300"
                [style.width.%]="usagePercentage()"
              ></div>
            </div>
          </div>

          <div class="bg-card rounded-2xl border border-border shadow-sm p-6 animate-stagger" style="--index:3">
            <p class="text-xs font-bold uppercase tracking-widest text-muted-foreground">Objects</p>
            <p class="text-2xl font-bold tracking-tight font-mono tabular-nums mt-2 text-foreground">
              {{ objects().length }}
            </p>
          </div>

        </div>

        <!-- ── Bucket management ─────────────────────────────────── -->
        <section class="bg-card rounded-2xl border border-border shadow-sm p-6">
          <div class="flex items-center justify-between mb-6">
            <div>
              <h2 class="text-xl font-semibold text-foreground">Buckets</h2>
              <p class="text-sm text-muted-foreground mt-0.5">Organise your files into buckets</p>
            </div>
          </div>

          <!-- Create bucket input -->
          <div class="flex gap-3 mb-6">
            <input
              type="text"
              [ngModel]="newBucketName()"
              (ngModelChange)="newBucketName.set($event)"
              placeholder="new-bucket-name"
              (keyup.enter)="createBucket()"
              class="flex-1 rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-foreground/40 transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background"
            />
            <button
              (click)="createBucket()"
              [disabled]="!newBucketName() || loading()"
              class="rounded-md bg-primary px-4 py-2 text-sm font-semibold text-primary-foreground shadow-sm transition-transform duration-200 hover:bg-primary/90 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 active:scale-[0.98] disabled:opacity-50 disabled:pointer-events-none whitespace-nowrap"
            >
              Create bucket
            </button>
          </div>

          <!-- Alert: error -->
          @if (error()) {
            <div class="mb-4 rounded-md bg-destructive/10 border border-destructive/20 px-4 py-3 flex items-start gap-2" role="alert">
              <svg class="size-4 mt-0.5 shrink-0 text-destructive" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                <path fill-rule="evenodd" d="M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0zm-8-5a.75.75 0 0 1 .75.75v4.5a.75.75 0 0 1-1.5 0v-4.5A.75.75 0 0 1 10 5zm0 10a1 1 0 1 0 0-2 1 1 0 0 0 0 2z" clip-rule="evenodd"/>
              </svg>
              <p class="text-sm text-destructive">{{ error() }}</p>
            </div>
          }

          <!-- Alert: success -->
          @if (success()) {
            <div class="mb-4 rounded-md bg-emerald-500/10 border border-emerald-500/20 px-4 py-3 flex items-start gap-2" role="status">
              <svg class="size-4 mt-0.5 shrink-0 text-emerald-500" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                <path fill-rule="evenodd" d="M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16zm3.857-9.809a.75.75 0 0 0-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 1 0-1.06 1.061l2.5 2.5a.75.75 0 0 0 1.137-.089l4-5.5z" clip-rule="evenodd"/>
              </svg>
              <p class="text-sm text-emerald-600 dark:text-emerald-400">{{ success() }}</p>
            </div>
          }

          <!-- Loading: skeleton -->
          @if (loading() && buckets().length === 0) {
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
              @for (_ of [1,2,3]; track $index) {
                <div class="h-24 rounded-2xl bg-muted animate-skeleton"></div>
              }
            </div>
          }

          <!-- Buckets grid -->
          @if (!loading() || buckets().length > 0) {
            @if (buckets().length > 0) {
              <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                @for (bucket of buckets(); track bucket.name) {
                  <button
                    (click)="selectBucket(bucket.name)"
                    class="group text-start rounded-2xl border-2 p-5 transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 active:scale-[0.98]"
                    [class]="currentBucket() === bucket.name
                      ? 'border-primary bg-primary/5'
                      : 'border-border bg-card hover:border-primary/40 hover:bg-muted/50'"
                    [attr.aria-pressed]="currentBucket() === bucket.name"
                  >
                    <!-- Folder icon -->
                    <svg class="size-8 text-muted-foreground group-hover:text-primary transition-colors duration-200"
                         [class.text-primary]="currentBucket() === bucket.name"
                         xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                      <path d="M19.5 21a3 3 0 0 0 3-3v-4.5a3 3 0 0 0-3-3h-15a3 3 0 0 0-3 3V18a3 3 0 0 0 3 3h15zM1.5 10.146V6a3 3 0 0 1 3-3h5.379a2.25 2.25 0 0 1 1.59.659l2.122 2.121c.14.141.331.22.53.22H19.5a3 3 0 0 1 3 3v1.146A4.483 4.483 0 0 0 19.5 9h-15a4.483 4.483 0 0 0-3 1.146z"/>
                    </svg>
                    <p class="text-sm font-semibold text-foreground mt-2 truncate">{{ bucket.name }}</p>
                    <p class="text-xs text-muted-foreground mt-0.5">{{ formatDate(bucket.created_at) }}</p>
                  </button>
                }
              </div>
            } @else if (!loading()) {
              <!-- Empty state -->
              <div class="py-16 text-center">
                <div class="mx-auto size-12 rounded-full bg-muted flex items-center justify-center mb-4">
                  <svg class="size-5 text-muted-foreground" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                    <path d="M19.5 21a3 3 0 0 0 3-3v-4.5a3 3 0 0 0-3-3h-15a3 3 0 0 0-3 3V18a3 3 0 0 0 3 3h15zM1.5 10.146V6a3 3 0 0 1 3-3h5.379a2.25 2.25 0 0 1 1.59.659l2.122 2.121c.14.141.331.22.53.22H19.5a3 3 0 0 1 3 3v1.146A4.483 4.483 0 0 0 19.5 9h-15a4.483 4.483 0 0 0-3 1.146z"/>
                  </svg>
                </div>
                <p class="text-sm font-medium text-foreground">No buckets yet</p>
                <p class="text-xs text-muted-foreground mt-1">Create a bucket above to get started</p>
              </div>
            }
          }
        </section>

        <!-- ── File manager ──────────────────────────────────────── -->
        @if (currentBucket()) {
          <section class="bg-card rounded-2xl border border-border shadow-sm p-6">

            <!-- Section header -->
            <div class="flex items-center justify-between mb-6">
              <div>
                <h2 class="text-xl font-semibold text-foreground">{{ currentBucket() }}</h2>
                <p class="text-sm text-muted-foreground mt-0.5">
                  {{ objects().length }} {{ objects().length === 1 ? 'file' : 'files' }}
                </p>
              </div>

              <!-- Upload button -->
              <div>
                <input
                  type="file"
                  #fileInput
                  (change)="onFileSelected($event)"
                  class="hidden"
                  aria-label="Choose file to upload"
                />
                <button
                  (click)="fileInput.click()"
                  [disabled]="loading()"
                  class="inline-flex items-center gap-2 rounded-md border border-border px-4 py-2 text-sm font-semibold text-foreground hover:bg-muted transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 active:scale-[0.98] disabled:opacity-50 disabled:pointer-events-none"
                >
                  <svg class="size-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                    <path d="M9.25 13.25a.75.75 0 0 0 1.5 0V4.636l2.955 3.129a.75.75 0 0 0 1.09-1.03l-4.25-4.5a.75.75 0 0 0-1.09 0l-4.25 4.5a.75.75 0 1 0 1.09 1.03L9.25 4.636v8.614z"/>
                    <path d="M3.5 12.75a.75.75 0 0 0-1.5 0v2.5A2.75 2.75 0 0 0 4.75 18h10.5A2.75 2.75 0 0 0 18 15.25v-2.5a.75.75 0 0 0-1.5 0v2.5c0 .69-.56 1.25-1.25 1.25H4.75c-.69 0-1.25-.56-1.25-1.25v-2.5z"/>
                  </svg>
                  Upload file
                </button>
              </div>
            </div>

            <!-- File manager error -->
            @if (fileError()) {
              <div class="mb-4 rounded-md bg-destructive/10 border border-destructive/20 px-4 py-3 flex items-start gap-2" role="alert">
                <svg class="size-4 mt-0.5 shrink-0 text-destructive" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                  <path fill-rule="evenodd" d="M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0zm-8-5a.75.75 0 0 1 .75.75v4.5a.75.75 0 0 1-1.5 0v-4.5A.75.75 0 0 1 10 5zm0 10a1 1 0 1 0 0-2 1 1 0 0 0 0 2z" clip-rule="evenodd"/>
                </svg>
                <p class="text-sm text-destructive">{{ fileError() }}</p>
              </div>
            }

            <!-- Loading: skeleton rows -->
            @if (loading() && objects().length === 0) {
              <div class="space-y-2">
                @for (_ of [1,2,3,4]; track $index) {
                  <div class="h-12 rounded-md bg-muted animate-skeleton"></div>
                }
              </div>
            }

            <!-- Files table -->
            @if (!loading() || objects().length > 0) {
              @if (objects().length > 0) {
                <div class="overflow-x-auto">
                  <table class="w-full">
                    <thead>
                      <tr class="border-b border-border">
                        <th scope="col" class="pb-3 text-start text-xs font-bold uppercase tracking-widest text-muted-foreground">Name</th>
                        <th scope="col" class="pb-3 text-end text-xs font-bold uppercase tracking-widest text-muted-foreground">Size</th>
                        <th scope="col" class="pb-3 text-end text-xs font-bold uppercase tracking-widest text-muted-foreground">Actions</th>
                      </tr>
                    </thead>
                    <tbody class="divide-y divide-border">
                      @for (obj of objects(); track obj.key) {
                        <tr class="group">
                          <td class="py-3 pe-4 text-sm text-foreground">{{ obj.key }}</td>
                          <td class="py-3 pe-4 text-sm font-mono tabular-nums text-foreground/60 text-end whitespace-nowrap">
                            {{ formatBytes(obj.size) }}
                          </td>
                          <td class="py-3 text-end">
                            <div class="flex items-center justify-end gap-2">
                              <button
                                (click)="downloadFile(obj.key)"
                                class="inline-flex items-center gap-1.5 rounded-md px-2.5 py-1 text-xs font-semibold text-foreground hover:bg-muted transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 active:scale-[0.98]"
                                aria-label="Download {{ obj.key }}"
                              >
                                <svg class="size-3.5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                                  <path d="M10.75 2.75a.75.75 0 0 0-1.5 0v8.614L6.295 8.235a.75.75 0 1 0-1.09 1.03l4.25 4.5a.75.75 0 0 0 1.09 0l4.25-4.5a.75.75 0 0 0-1.09-1.03l-2.955 3.129V2.75z"/>
                                  <path d="M3.5 12.75a.75.75 0 0 0-1.5 0v2.5A2.75 2.75 0 0 0 4.75 18h10.5A2.75 2.75 0 0 0 18 15.25v-2.5a.75.75 0 0 0-1.5 0v2.5c0 .69-.56 1.25-1.25 1.25H4.75c-.69 0-1.25-.56-1.25-1.25v-2.5z"/>
                                </svg>
                                Download
                              </button>
                              <button
                                (click)="deleteFile(obj.key)"
                                class="inline-flex items-center gap-1.5 rounded-md px-2.5 py-1 text-xs font-semibold text-destructive hover:bg-destructive/10 transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 active:scale-[0.98]"
                                aria-label="Delete {{ obj.key }}"
                              >
                                <svg class="size-3.5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                                  <path fill-rule="evenodd" d="M8.75 1A2.75 2.75 0 0 0 6 3.75v.443c-.795.077-1.584.176-2.365.298a.75.75 0 1 0 .23 1.482l.149-.022.841 10.518A2.75 2.75 0 0 0 7.596 19h4.807a2.75 2.75 0 0 0 2.742-2.53l.841-10.52.149.023a.75.75 0 0 0 .23-1.482A41.03 41.03 0 0 0 14 4.193V3.75A2.75 2.75 0 0 0 11.25 1h-2.5zM10 4c.84 0 1.673.025 2.5.075V3.75c0-.69-.56-1.25-1.25-1.25h-2.5c-.69 0-1.25.56-1.25 1.25v.325C8.327 4.025 9.16 4 10 4zM8.58 7.72a.75.75 0 0 0-1.5.06l.3 7.5a.75.75 0 1 0 1.5-.06l-.3-7.5zm4.34.06a.75.75 0 1 0-1.5-.06l-.3 7.5a.75.75 0 1 0 1.5.06l.3-7.5z" clip-rule="evenodd"/>
                                </svg>
                                Delete
                              </button>
                            </div>
                          </td>
                        </tr>
                      }
                    </tbody>
                  </table>
                </div>
              } @else if (!loading()) {
                <!-- Empty state -->
                <div class="py-16 text-center">
                  <div class="mx-auto size-12 rounded-full bg-muted flex items-center justify-center mb-4">
                    <svg class="size-5 text-muted-foreground" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                      <path d="M19.906 9c.382 0 .749.057 1.094.162V9a3 3 0 0 0-3-3h-3.879a.75.75 0 0 1-.53-.22L11.47 3.66A2.25 2.25 0 0 0 9.879 3H6a3 3 0 0 0-3 3v3.162A3.756 3.756 0 0 1 4.094 9h15.812zM4.094 10.5a2.25 2.25 0 0 0-2.227 2.568l.857 6A2.25 2.25 0 0 0 4.951 21H19.05a2.25 2.25 0 0 0 2.227-1.932l.857-6a2.25 2.25 0 0 0-2.227-2.568H4.094z"/>
                    </svg>
                  </div>
                  <p class="text-sm font-medium text-foreground">This bucket is empty</p>
                  <p class="text-xs text-muted-foreground mt-1">Upload a file to get started</p>
                </div>
              }
            }
          </section>
        }

      </main>
    </div>
  `,
  styles: [`
    @keyframes shimmer {
      0%, 100% { opacity: 1; }
      50%       { opacity: 0.4; }
    }
    .animate-skeleton { animation: shimmer 1.5s ease-in-out infinite; }
    @keyframes fadeUp {
      from { opacity: 0; transform: translateY(8px); }
      to   { opacity: 1; transform: translateY(0); }
    }
    .animate-stagger {
      animation: fadeUp 200ms cubic-bezier(0.16, 1, 0.3, 1) both;
      animation-delay: calc(var(--index, 0) * 50ms);
    }
  `]
})
export class DashboardComponent implements OnInit {
  private authService = inject(AuthService);
  private storageService = inject(StorageService);
  private router = inject(Router);

  user = signal<User | null>(null);
  currentBucket = signal('');
  buckets = signal<Bucket[]>([]);
  newBucketName = signal('');
  objects = signal<S3Object[]>([]);
  loading = signal(false);
  error = signal('');
  success = signal('');
  fileError = signal('');

  usagePercentage = computed(() => {
    const user = this.user();
    if (!user || user.storage_quota === 0) return 0;
    return Math.round((user.storage_used / user.storage_quota) * 100);
  });

  ngOnInit(): void {
    this.authService.currentUser$.subscribe(user => this.user.set(user));

    if (!this.authService.isAuthenticated()) {
      this.router.navigate(['/login']);
      return;
    }

    if (this.user()) {
      this.loadBuckets();
    }

    this.authService.getCurrentUser().subscribe({
      next: () => {
        if (!this.buckets().length) this.loadBuckets();
      },
      error: (err: any) => {
        if (err.status === 401) {
          this.authService.logout();
          this.router.navigate(['/login']);
        }
      }
    });
  }

  formatBytes(bytes: number): string {
    return this.storageService.formatBytes(bytes);
  }

  formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString(undefined, { dateStyle: 'medium' });
  }

  loadBuckets(): void {
    this.loading.set(true);
    this.error.set('');

    this.storageService.listBuckets().subscribe({
      next: (response) => {
        this.buckets.set(response.buckets);
        this.loading.set(false);
      },
      error: () => {
        this.error.set('Failed to load buckets. Check your connection and try again.');
        this.loading.set(false);
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
        this.success.set(`Bucket "${bucket.name}" created.`);
        this.newBucketName.set('');
        this.loadBuckets();
        this.loading.set(false);
      },
      error: () => {
        this.error.set('Failed to create bucket. Names must be lowercase with no spaces.');
        this.loading.set(false);
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
    this.fileError.set('');

    this.storageService.listBucketObjects(this.currentBucket()).subscribe({
      next: (response: ListBucketResponse) => {
        this.objects.set(response.objects || []);
        this.loading.set(false);
      },
      error: () => {
        this.fileError.set('Failed to load files from this bucket.');
        this.loading.set(false);
      }
    });
  }

  onFileSelected(event: any): void {
    const file: File = event.target.files[0];
    if (!file || !this.currentBucket()) return;

    this.loading.set(true);
    this.fileError.set('');
    this.success.set('');

    this.storageService.uploadFile(this.currentBucket(), file.name, file).subscribe({
      next: () => {
        this.success.set(`"${file.name}" uploaded.`);
        this.loading.set(false);
        this.loadBucket();
      },
      error: () => {
        this.fileError.set('Upload failed. Check the file size and try again.');
        this.loading.set(false);
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
      error: () => this.fileError.set('Download failed.')
    });
  }

  deleteFile(key: string): void {
    if (!confirm(`Permanently delete "${key}"? This cannot be undone.`)) return;

    this.storageService.deleteFile(this.currentBucket(), key).subscribe({
      next: () => {
        this.success.set(`"${key}" deleted.`);
        this.loadBucket();
      },
      error: () => this.fileError.set('Failed to delete file.')
    });
  }

  logout(): void {
    this.authService.logout();
    this.router.navigate(['/login']);
  }
}
