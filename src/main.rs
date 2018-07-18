/*
by navewindre

never coded in rust before so this is really messy
*/

extern crate winapi;
extern crate kernel32;
extern crate user32;

use kernel32 as k32;

use std::ptr::null_mut as nullptr;
use std::ffi::CString;

use winapi::DWORD as ulong_t;

fn key_state( key: i32 ) -> bool {
    unsafe {
        let state = user32::GetAsyncKeyState( key );

        return ( state & 0x8000 ) != 0;
    }
}

fn find_process_by_window( title: &str ) -> ulong_t {
    let _title = CString::new( title ).unwrap( );
    
    unsafe {
        let wnd_handle = user32::FindWindowA( nullptr( ), _title.as_ptr( ) );

        let mut pid = 0 as u32;
        user32::GetWindowThreadProcessId( wnd_handle, &mut pid as *mut u32 );

        return pid;
    }
}

pub struct process_t {
    m_handle: winapi::HANDLE,
    m_pid:    ulong_t
}

pub struct module_t {
    m_process: *mut process_t,
    m_base:         ulong_t
}

impl process_t {
    pub fn find_by_window( &mut self, window: &str ) -> bool {
        self.m_pid = find_process_by_window( window );
        return self.m_pid != 0;
    }

    pub fn open( &mut self ) -> bool {
        if self.m_pid == 0 {
            return false;
        }

        unsafe {
            self.m_handle = k32::OpenProcess( 
                0x1f0fff as ulong_t, 
                winapi::FALSE, 
                self.m_pid );
        }

        return self.m_handle != nullptr( );
    }

    pub fn close( &mut self ) {
        if self.m_handle != nullptr( ) {
            unsafe { k32::CloseHandle( self.m_handle ) };
        }
    }

    pub fn read< t: Default >( &mut self, address: u32 ) -> t {
        let mut ret: t = Default::default( );

        unsafe {
            k32::ReadProcessMemory( 
                self.m_handle, 
                address as *const winapi::c_void, 
                &mut ret as *mut t as *mut winapi::c_void, 
                std::mem::size_of::< t >( ) as u32,
                nullptr( ) );

            return ret;
        }
    }

    pub fn write< t: Default >( &mut self, address: u32, mut value: t ) {
        unsafe {
            k32::WriteProcessMemory(
                self.m_handle,
                address as *mut winapi::c_void,
                &mut value as *mut t as *const winapi::c_void,
                std::mem::size_of::< t >( ) as u32,
                nullptr( )
            );
        }
    }
}

impl module_t {
    pub fn find_by_name( &mut self, name: &str ) -> bool {
        use winapi::tlhelp32::MODULEENTRY32;

        unsafe {
            let snapshot = k32::CreateToolhelp32Snapshot( 
                0x8 as ulong_t,
                ( *self.m_process ).m_pid
            );

            let mut module = MODULEENTRY32{
                dwSize: std::mem::size_of::< MODULEENTRY32 >( ) as u32,
                th32ModuleID: 0,
                th32ProcessID: 0,
                GlblcntUsage: 0,
                ProccntUsage: 0,
                modBaseAddr: nullptr( ),
                modBaseSize: 0,
                hModule: nullptr( ),
                szModule: [ 0; 256 ],
                szExePath: [ 0; 260 ]
            };

            k32::Module32First( snapshot, &mut module as *mut MODULEENTRY32 );

            loop {
                //rust winapi fucking sucks
                let _u8slice = &*( &mut module.szModule[ .. ] as *mut [ i8 ] as *mut [ u8 ] );
                let module_name = std::str::from_utf8( _u8slice ).unwrap( );

                if module_name.find( name ) == Some( 0 ) {
                    self.m_base = module.modBaseAddr as ulong_t;
                    return true;
                }

                if k32::Module32Next( snapshot, &mut module as *mut MODULEENTRY32 ) == winapi::FALSE {
                    return false; 
                }
            }
        }

        return false;
    }
}

fn main( ) {
    const VK_SPACE: i32 = 32;

    let mut process = process_t {
        m_handle: nullptr( ),
        m_pid: 0
    };

    let mut client_dll = module_t {
        m_process: &mut process as *mut process_t,
        m_base: 0
    };

    loop {
        if process.find_by_window( "Counter-Strike: Global Offensive" ) {
            process.open( );

            if client_dll.find_by_name( "client.dll" ) {
                break;
            }
        }
    }

    println!( "attached to process, id: {}", process.m_pid );
    println!( "client.dll: {}", client_dll.m_base );

    loop {
        const LOCALPLAYER: u32 = 0xAB6D4C;
        const FLAGS:       u32 = 0x100;
        const FORCEJUMP:   u32 = 0x4F2C870;

        let local_player = process.read::< u32 >( client_dll.m_base + LOCALPLAYER );
        let flags        = process.read::< i32 >( local_player + FLAGS );
        
        if key_state( VK_SPACE ) && ( flags & 1 ) != 0 {
            process.write::< i32 >( client_dll.m_base + FORCEJUMP, 6 );
        }
    }
}