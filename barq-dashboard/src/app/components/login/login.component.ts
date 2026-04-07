import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { AuthService } from '../../services/auth.service';

@Component({
  selector: 'app-login',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="login-container">
      <div class="login-card">
        <h1>⚡ BARQ X30</h1>
        <p class="subtitle">Ultra-High Performance Object Storage</p>
        
        <form (ngSubmit)="onSubmit()" #loginForm="ngForm">
          <div class="form-group">
            <label for="email">Email</label>
            <input 
              type="email" 
              id="email" 
              name="email"
              [(ngModel)]="email" 
              required 
              email
              placeholder="your@email.com"
            />
          </div>

          <div class="form-group">
            <label for="password">Password</label>
            <input 
              type="password" 
              id="password" 
              name="password"
              [(ngModel)]="password" 
              required
              placeholder="••••••••"
            />
          </div>

          <div class="error" *ngIf="error">{{ error }}</div>

          <button type="submit" [disabled]="!loginForm.form.valid || loading">
            {{ loading ? 'Signing in...' : 'Sign In' }}
          </button>
        </form>

        <div class="stats">
          <div class="stat">
            <span class="stat-value">30μs</span>
            <span class="stat-label">Target Latency</span>
          </div>
          <div class="stat">
            <span class="stat-value">1000x</span>
            <span class="stat-label">Faster than S3</span>
          </div>
        </div>
      </div>
    </div>
  `,
  styles: [`
    .login-container {
      min-height: 100vh;
      display: flex;
      align-items: center;
      justify-content: center;
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      padding: 20px;
    }

    .login-card {
      background: white;
      padding: 40px;
      border-radius: 12px;
      box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
      max-width: 400px;
      width: 100%;
    }

    h1 {
      margin: 0 0 10px 0;
      text-align: center;
      color: #333;
      font-size: 36px;
    }

    .subtitle {
      text-align: center;
      color: #666;
      margin: 0 0 30px 0;
      font-size: 14px;
    }

    .form-group {
      margin-bottom: 20px;
    }

    label {
      display: block;
      margin-bottom: 5px;
      color: #333;
      font-weight: 500;
    }

    input {
      width: 100%;
      padding: 12px;
      border: 1px solid #ddd;
      border-radius: 6px;
      font-size: 14px;
      box-sizing: border-box;
    }

    input:focus {
      outline: none;
      border-color: #667eea;
    }

    button {
      width: 100%;
      padding: 12px;
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      color: white;
      border: none;
      border-radius: 6px;
      font-size: 16px;
      font-weight: 600;
      cursor: pointer;
      transition: transform 0.2s;
    }

    button:hover:not(:disabled) {
      transform: translateY(-2px);
    }

    button:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }

    .error {
      color: #e74c3c;
      margin: 15px 0;
      text-align: center;
      font-size: 14px;
    }

    .stats {
      display: flex;
      justify-content: space-around;
      margin-top: 30px;
      padding-top: 30px;
      border-top: 1px solid #eee;
    }

    .stat {
      text-align: center;
    }

    .stat-value {
      display: block;
      font-size: 24px;
      font-weight: bold;
      color: #667eea;
    }

    .stat-label {
      display: block;
      font-size: 12px;
      color: #666;
      margin-top: 5px;
    }
  `]
})
export class LoginComponent {
  email = '';
  password = '';
  error = '';
  loading = false;

  constructor(
    private authService: AuthService,
    private router: Router
  ) {}

  onSubmit(): void {
    this.loading = true;
    this.error = '';

    this.authService.login(this.email, this.password).subscribe({
      next: () => {
        this.router.navigate(['/dashboard']);
      },
      error: (err) => {
        this.error = 'Invalid email or password';
        this.loading = false;
        console.error('Login error:', err);
      }
    });
  }
}
