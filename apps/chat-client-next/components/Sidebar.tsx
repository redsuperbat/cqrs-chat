"use client";

import { useRouter } from "next/router";
import { FC, useState } from "react";
import { GetChatsData } from "../pages/api/chats";
import { UserStore } from "../storage/user-store";
import { useSwr } from "../swr/use-swr";

export const Sidebar: FC = () => {
  const [isOpen, setIsOpen] = useState(false);
  const { data } = useSwr<GetChatsData>(
    `/api/chats?user_id=${UserStore.get()?.hashedUsername}`
  );
  const router = useRouter();

  const toggleSidebar = () => setIsOpen(!isOpen);

  const ToggleBtn: FC = () => (
    <div onClick={toggleSidebar} className="cursor-pointer z-10">
      <svg
        height="24px"
        width="24px"
        viewBox="0 0 24 24"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <path
          d="M4 6H20M4 12H20M4 18H11"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    </div>
  );

  const FlatButton: FC<{ text: string; chat_id: string }> = ({
    text,
    chat_id,
  }) => {
    const onClick = async () => {
      await router.push(`/chats/${chat_id}`);
      toggleSidebar();
    };
    return (
      <button
        className="border-2 border-white rounded-md w-full p-1 mb-2 hover:bg-slate-400 hover:bg-opacity-30 transition-all"
        onClick={onClick}
      >
        <div className="border border-white rounded p-2 ">
          <div>{text}</div>
        </div>
      </button>
    );
  };

  return (
    <>
      <div className="absolute top-2 left-2 z-10">
        <ToggleBtn />
      </div>
      <div
        className={`${
          isOpen ? "opacity-100" : "-translate-x-80 opacity-0"
        } fixed bg-gray-800 top-0 w-80 left-0 h-full z-50 transition-all`}
      >
        <div className="flex justify-end p-2 text-white">
          <ToggleBtn />
        </div>
        <div className="p-2 flex flex-col text-white">
          <h1 className="text-xl mb-3 underline"> Previous chatty endeavors</h1>
          {(data?.chats ?? []).map((it) => (
            <FlatButton
              key={it.chat_id}
              chat_id={it.chat_id}
              text={it.subject}
            />
          ))}
        </div>
      </div>
    </>
  );
};
