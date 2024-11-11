bump VERSION="patch":
    ./scripts/bump-version --{{VERSION}}

preview-site:
    uvx --with mkdocs-material,markdown-include mkdocs serve

publish-site:
    git push -f origin HEAD:docs

release:
    ./scripts/release.sh

install:
    ./scripts/install.sh

install-windows:
    ./scripts/install.ps1
