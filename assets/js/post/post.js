  document.addEventListener('DOMContentLoaded', function() {
    const tocList = document.getElementById('toc-list');
    const headings = document.querySelectorAll('.prose h2, .prose h3');
    const mainContent = document.querySelector('.main-content');
    
    if (headings.length === 0 || !tocList) return;
    
    headings.forEach((heading, index) => {
      if (!heading.id) {
        const headingText = heading.textContent.trim();
        heading.id = headingText
          .toLowerCase()
          .replace(/[^\w\s-]/g, '')
          .replace(/\s+/g, '-');
        heading.id += `-${index}`;
      }
      
      const listItem = document.createElement('li');
      const link = document.createElement('a');
      link.href = `#${heading.id}`;
      link.textContent = heading.textContent.trim();
      link.setAttribute('data-heading-id', heading.id);
      
      if (heading.tagName.toLowerCase() === 'h3') {
        link.classList.add('toc-h3');
      }
      
      listItem.appendChild(link);
      tocList.appendChild(listItem);
    });
    
    const tocLinks = document.querySelectorAll('.toc-list a');
    
    const highlightTocOnScroll = () => {
      const scrollPos = window.scrollY + 100;
      let currentHeadingId = null;
      
      headings.forEach(heading => {
        const headingTop = heading.getBoundingClientRect().top + window.scrollY;
        
        if (headingTop <= scrollPos) {
          currentHeadingId = heading.id;
        }
      });
      
      tocLinks.forEach(link => {
        link.classList.remove('active');
      });
      
      if (currentHeadingId) {
        const activeLink = document.querySelector(`.toc-list a[data-heading-id="${currentHeadingId}"]`);
        if (activeLink) {
          activeLink.classList.add('active');
        }
      }
    };
    
    tocLinks.forEach(link => {
      link.addEventListener('click', (e) => {
        e.preventDefault();
        const targetId = link.getAttribute('href').substring(1);
        const targetElement = document.getElementById(targetId);
        
        if (targetElement) {
          window.scrollTo({
            top: targetElement.offsetTop - 20,
            behavior: 'smooth'
          });
          
          history.pushState(null, null, `#${targetId}`);
          
          tocLinks.forEach(l => l.classList.remove('active'));
          link.classList.add('active');
        }
      });
    });
    
    window.addEventListener('scroll', highlightTocOnScroll);
    
    highlightTocOnScroll();
    
    const codeBlocks = document.querySelectorAll('pre code');
    
    codeBlocks.forEach((codeBlock, index) => {
      const pre = codeBlock.parentElement;
      
      const copyButton = document.createElement('button');
      copyButton.className = 'copy-button';
      copyButton.setAttribute('aria-label', 'Copy code');
      copyButton.setAttribute('data-index', index);
      
      copyButton.innerHTML = `
        <svg class="copy-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
        </svg>
        <svg class="check-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="20 6 9 17 4 12"></polyline>
        </svg>
        <span>Copy</span>
      `;
      
      pre.appendChild(copyButton);
      
      copyButton.addEventListener('click', () => {
        const code = codeBlock.textContent;
        
        navigator.clipboard.writeText(code).then(() => {
          copyButton.classList.add('copied');
          copyButton.querySelector('span').textContent = 'Copied!';
          
          setTimeout(() => {
            copyButton.classList.remove('copied');
            copyButton.querySelector('span').textContent = 'Copy';
          }, 2000);
        }).catch(err => {
          console.error('Failed to copy code: ', err);
          copyButton.querySelector('span').textContent = 'Failed';
          
          setTimeout(() => {
            copyButton.querySelector('span').textContent = 'Copy';
          }, 2000);
        });
      });
    });
  });
