import { FC, ReactNode, useState } from "react";

export const SideBar: FC<{ children: ReactNode }> = ({ children }) => {
  const toggleSidebar = () => {
    const sidebar = document.querySelector(".sidebar");
    const btn = document.querySelector(".menu-button");
    sidebar?.classList.toggle("active");
    btn?.classList.toggle("open");
  };

  const HamburgerMenu = () => {
    const [isOpen, setIsOpen] = useState(false);

    function toggleMenu() {
      setIsOpen(!isOpen);
    }

    return (
      <button className={`menu-btn ${isOpen && "open"}`} onClick={toggleMenu}>
        <div className="bar"></div>
        <div className="bar"></div>
        <div className="bar"></div>
      </button>
    );
  };

  return (
    <div>
      <HamburgerMenu />
      <div className="sidebar">
        <div className="sidebar-content">
          <HamburgerMenu />
          {children}
        </div>
      </div>
    </div>
  );
};
