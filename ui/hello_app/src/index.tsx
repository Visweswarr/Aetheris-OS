import React from 'react';
import ReactDOM from 'react-dom/client';

/**
 * Hello Polymera App - Minimal TypeScript React Application
 * 
 * This is a minimal UI stub that demonstrates the basic structure
 * for Polymera OS user interface components. It renders a simple
 * "Hello Polymera" message as required by the epic specification.
 */

// Main App Component
const HelloPolymeraApp: React.FC = () => {
  return (
    <div className="hello-polymera-app">
      <header className="app-header">
        <h1>🚀 Hello Polymera</h1>
        <p>Welcome to the future of operating systems</p>
      </header>
      
      <main className="app-main">
        <section className="hero-section">
          <h2>Polymera OS</h2>
          <p>A next-generation operating system built for security, performance, and privacy.</p>
          
          <div className="features-grid">
            <div className="feature-card">
              <h3>🔐 Security First</h3>
              <p>Post-quantum cryptography and zero-knowledge proofs</p>
            </div>
            
            <div className="feature-card">
              <h3>⚡ High Performance</h3>
              <p>Deterministic execution and optimized resource management</p>
            </div>
            
            <div className="feature-card">
              <h3>🛡️ Privacy by Design</h3>
              <p>Built-in privacy controls and data protection</p>
            </div>
            
            <div className="feature-card">
              <h3>🌐 Web-Native</h3>
              <p>WebAssembly runtime and modern web technologies</p>
            </div>
          </div>
        </section>
        
        <section className="status-section">
          <h3>System Status</h3>
          <div className="status-indicators">
            <div className="status-item">
              <span className="status-dot status-ok"></span>
              <span>Kernel: Running</span>
            </div>
            <div className="status-item">
              <span className="status-dot status-ok"></span>
              <span>Security: Active</span>
            </div>
            <div className="status-item">
              <span className="status-dot status-ok"></span>
              <span>UI: Ready</span>
            </div>
          </div>
        </section>
      </main>
      
      <footer className="app-footer">
        <p>&copy; 2025 Polymera OS. Built with React, TypeScript, and Bazel.</p>
      </footer>
    </div>
  );
};

// App Styles (inline for minimal setup)
const appStyles = `
  .hello-polymera-app {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
    color: white;
  }
  
  .app-header {
    text-align: center;
    margin-bottom: 40px;
    padding: 40px 0;
  }
  
  .app-header h1 {
    font-size: 3rem;
    margin: 0 0 10px 0;
    font-weight: 700;
    text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
  }
  
  .app-header p {
    font-size: 1.2rem;
    margin: 0;
    opacity: 0.9;
  }
  
  .app-main {
    margin-bottom: 40px;
  }
  
  .hero-section {
    text-align: center;
    margin-bottom: 60px;
  }
  
  .hero-section h2 {
    font-size: 2.5rem;
    margin: 0 0 20px 0;
    font-weight: 600;
  }
  
  .hero-section p {
    font-size: 1.3rem;
    margin: 0 0 40px 0;
    opacity: 0.9;
  }
  
  .features-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 30px;
    margin-top: 40px;
  }
  
  .feature-card {
    background: rgba(255, 255, 255, 0.1);
    padding: 30px;
    border-radius: 15px;
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    transition: transform 0.3s ease, box-shadow 0.3s ease;
  }
  
  .feature-card:hover {
    transform: translateY(-5px);
    box-shadow: 0 10px 30px rgba(0,0,0,0.3);
  }
  
  .feature-card h3 {
    font-size: 1.5rem;
    margin: 0 0 15px 0;
    font-weight: 600;
  }
  
  .feature-card p {
    font-size: 1rem;
    margin: 0;
    opacity: 0.8;
    line-height: 1.5;
  }
  
  .status-section {
    background: rgba(255, 255, 255, 0.1);
    padding: 30px;
    border-radius: 15px;
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.2);
  }
  
  .status-section h3 {
    font-size: 1.5rem;
    margin: 0 0 20px 0;
    text-align: center;
    font-weight: 600;
  }
  
  .status-indicators {
    display: flex;
    justify-content: center;
    gap: 40px;
    flex-wrap: wrap;
  }
  
  .status-item {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 1.1rem;
  }
  
  .status-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    display: inline-block;
  }
  
  .status-ok {
    background: #4ade80;
    box-shadow: 0 0 10px rgba(74, 222, 128, 0.5);
  }
  
  .status-warning {
    background: #fbbf24;
    box-shadow: 0 0 10px rgba(251, 191, 36, 0.5);
  }
  
  .status-error {
    background: #f87171;
    box-shadow: 0 0 10px rgba(248, 113, 113, 0.5);
  }
  
  .app-footer {
    text-align: center;
    padding: 30px 0;
    border-top: 1px solid rgba(255, 255, 255, 0.2);
    opacity: 0.8;
  }
  
  .app-footer p {
    margin: 0;
    font-size: 1rem;
  }
  
  @media (max-width: 768px) {
    .hello-polymera-app {
      padding: 15px;
    }
    
    .app-header h1 {
      font-size: 2rem;
    }
    
    .hero-section h2 {
      font-size: 1.8rem;
    }
    
    .features-grid {
      grid-template-columns: 1fr;
      gap: 20px;
    }
    
    .status-indicators {
      flex-direction: column;
      align-items: center;
      gap: 20px;
    }
  }
`;

// Inject styles
const styleElement = document.createElement('style');
styleElement.textContent = appStyles;
document.head.appendChild(styleElement);

// Render the app
const rootElement = document.getElementById('root');
if (rootElement) {
  const root = ReactDOM.createRoot(rootElement);
  root.render(<HelloPolymeraApp />);
} else {
  console.error('Root element not found. Make sure there is an element with id="root" in your HTML.');
}

// Export for testing and external use
export default HelloPolymeraApp;
export { HelloPolymeraApp };

// Development helpers
if (process.env.NODE_ENV === 'development') {
  console.log('🚀 Hello Polymera App initialized in development mode');
  console.log('📱 App version:', '0.1.0');
  console.log('🔧 Build system:', 'Bazel + TypeScript');
  console.log('⚛️ React version:', React.version);
}
