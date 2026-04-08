import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { AuthService } from '../../services/auth.service';

@Component({
  selector: 'app-login',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [FormsModule],
  template: `
    <div class="min-h-[100dvh] bg-background flex">

      <!-- Left panel — branding (hidden on mobile) -->
      <div class="hidden lg:flex lg:w-1/2 flex-col justify-between p-12 bg-muted border-e border-border">
        <p class="text-sm font-semibold tracking-tight text-foreground">BARQ X30</p>

        <div>
          <h1 class="text-4xl font-extrabold tracking-tighter text-foreground" style="text-wrap: balance">
            Ultra-high performance<br>object storage.
          </h1>
          <p class="mt-4 text-sm text-muted-foreground max-w-[55ch]">
            Built for teams that need fast, reliable file storage with fine-grained access control.
          </p>
        </div>

        <div class="flex gap-10">
          <div>
            <p class="text-2xl font-bold tracking-tight font-mono tabular-nums text-foreground">30μs</p>
            <p class="text-xs font-bold uppercase tracking-widest text-muted-foreground mt-1">Target latency</p>
          </div>
          <div>
            <p class="text-2xl font-bold tracking-tight font-mono tabular-nums text-foreground">1000×</p>
            <p class="text-xs font-bold uppercase tracking-widest text-muted-foreground mt-1">Faster than S3</p>
          </div>
        </div>
      </div>

      <!-- Right panel — sign-in form -->
      <div class="w-full lg:w-1/2 flex items-center justify-center p-8">
        <div class="w-full max-w-sm animate-fade-up">

          <!-- Mobile logo -->
          <p class="text-sm font-semibold text-foreground mb-8 lg:hidden">BARQ X30</p>

          <div class="mb-8">
            <h2 class="text-2xl font-bold tracking-tight text-foreground">Sign in</h2>
            <p class="text-sm text-muted-foreground mt-1">Enter your credentials to access the dashboard.</p>
          </div>

          <form (ngSubmit)="onSubmit()" class="space-y-5">

            <!-- Email -->
            <div class="space-y-1.5">
              <label for="email" class="text-sm font-semibold text-foreground">Email</label>
              <input
                type="email"
                id="email"
                name="email"
                [(ngModel)]="email"
                required
                email
                placeholder="you@company.com"
                class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-foreground/40 transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background"
              />
            </div>

            <!-- Password -->
            <div class="space-y-1.5">
              <label for="password" class="text-sm font-semibold text-foreground">Password</label>
              <input
                type="password"
                id="password"
                name="password"
                [(ngModel)]="password"
                required
                placeholder="••••••••"
                class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-foreground/40 transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background"
              />
            </div>

            <!-- Inline error alert -->
            @if (error()) {
              <div class="rounded-md bg-destructive/10 border border-destructive/20 px-4 py-3 flex items-start gap-2" role="alert">
                <svg class="size-4 mt-0.5 shrink-0 text-destructive" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                  <path fill-rule="evenodd" d="M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0zm-8-5a.75.75 0 0 1 .75.75v4.5a.75.75 0 0 1-1.5 0v-4.5A.75.75 0 0 1 10 5zm0 10a1 1 0 1 0 0-2 1 1 0 0 0 0 2z" clip-rule="evenodd"/>
                </svg>
                <p class="text-sm text-destructive">{{ error() }}</p>
              </div>
            }

            <!-- Primary submit button — one per view -->
            <button
              type="submit"
              [disabled]="loading()"
              class="w-full rounded-md bg-primary px-4 py-2.5 text-sm font-semibold text-primary-foreground shadow-sm transition-transform duration-200 hover:bg-primary/90 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 active:scale-[0.98] disabled:opacity-50 disabled:pointer-events-none"
            >
              @if (loading()) {
                <span class="flex items-center justify-center gap-2">
                  <svg class="size-4 animate-spin" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" aria-hidden="true">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"/>
                  </svg>
                  Signing in…
                </span>
              } @else {
                Sign in
              }
            </button>

          </form>
        </div>
      </div>

    </div>
  `,
  styles: [`
    @keyframes spin { to { transform: rotate(360deg); } }
    .animate-spin { animation: spin 600ms linear infinite; }
  `]
})
export class LoginComponent {
  private authService = inject(AuthService);
  private router = inject(Router);

  email = '';
  password = '';
  error = signal('');
  loading = signal(false);

  onSubmit(): void {
    this.loading.set(true);
    this.error.set('');

    this.authService.login(this.email, this.password).subscribe({
      next: () => this.router.navigate(['/dashboard']),
      error: () => {
        this.error.set('Invalid email or password. Please try again.');
        this.loading.set(false);
      }
    });
  }
}

