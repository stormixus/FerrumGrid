document.addEventListener('DOMContentLoaded', () => {
  // --- Theme Toggle Logic ---
  const themeSwitch = document.querySelector('.theme-switch');
  const themeIcon = themeSwitch.querySelector('span');
  const ferrumgridDockIcon = document.getElementById('ferrumgrid-dock-icon');
  
  // Set default theme from localStorage or system preference
  const systemPrefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  const savedTheme = localStorage.getItem('theme');
  const activeTheme = savedTheme || (systemPrefersDark ? 'dark' : 'light');
  
  setTheme(activeTheme);

  themeSwitch.addEventListener('click', () => {
    const currentTheme = document.documentElement.getAttribute('data-theme');
    const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
  });

  function setTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('theme', theme);
    
    // Update theme switch icons (using simple unicode symbols for standard compatibility)
    if (theme === 'dark') {
      themeIcon.textContent = '☀️'; // Show sun when dark to switch to light
      if (ferrumgridDockIcon) ferrumgridDockIcon.src = 'assets/app-icon.png';
    } else {
      themeIcon.textContent = '🌙'; // Show moon when light to switch to dark
      if (ferrumgridDockIcon) ferrumgridDockIcon.src = 'assets/app-icon-light.png';
    }
  }

  // --- Copy Install Code Command ---
  const copyBtn = document.getElementById('btn-copy-install');
  const copyCodeText = document.getElementById('copy-code-text');
  
  if (copyBtn && copyCodeText) {
    copyBtn.addEventListener('click', () => {
      const codeToCopy = copyCodeText.textContent;
      navigator.clipboard.writeText(codeToCopy).then(() => {
        // Simple visual feedback
        const originalHTML = copyBtn.innerHTML;
        copyBtn.innerHTML = '⚡ <span style="font-size: 0.75rem; font-family: sans-serif; font-weight: 600; margin-left: 2px;">Copied!</span>';
        copyBtn.style.color = '#3ECF8E';
        setTimeout(() => {
          copyBtn.innerHTML = originalHTML;
          copyBtn.style.color = '';
        }, 1800);
      }).catch(err => {
        console.error('Failed to copy text: ', err);
      });
    });
  }

  // --- Feature Tabs Controller ---
  const tabButtons = document.querySelectorAll('.tab-btn');
  const tabContentBlocks = document.querySelectorAll('.tab-panel-block');

  tabButtons.forEach(button => {
    button.addEventListener('click', () => {
      const targetTabId = button.getAttribute('data-tab');

      // Deactivate all buttons & blocks
      tabButtons.forEach(btn => btn.classList.remove('active'));
      tabContentBlocks.forEach(block => block.style.display = 'none');

      // Activate clicked button and matching block
      button.classList.add('active');
      const targetBlock = document.getElementById(`tab-content-${targetTabId}`);
      if (targetBlock) {
        targetBlock.style.display = 'block';
      }
    });
  });

  // Set default tab on load (SQL Query Editor)
  const defaultTabBtn = document.querySelector('.tab-btn[data-tab="query"]');
  if (defaultTabBtn) defaultTabBtn.click();

  // --- Interactive Mock macOS Dock Bouncing Effect ---
  const dockItems = document.querySelectorAll('.dock-item');
  dockItems.forEach(item => {
    item.addEventListener('click', () => {
      const img = item.querySelector('img');
      if (img) {
        // Bounce animation triggers
        img.style.transition = 'transform 0.15s ease-out';
        img.style.transform = 'translateY(-30px) scale(1.1)';
        
        setTimeout(() => {
          img.style.transition = 'transform 0.35s cubic-bezier(0.175, 0.885, 0.32, 1.275)';
          img.style.transform = 'translateY(0) scale(1)';
        }, 180);
      }
    });
  });
});
