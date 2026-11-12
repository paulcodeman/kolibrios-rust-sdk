format ELF

include "src/func_constants.inc"

section '.text'
    public _exit
    public _start_window_draw
    public _end_window_draw
    public _define_window
    public _put_pixel
    public _display_message
    public _wait_event
    public _pressed_key
    public _define_button
    public _debug_write
    public _get_button_id
    public _init_heap
    public _alloc
    public _free
    public _get_lang
    public _load_dll
    public _set_event_mask

_exit:
    mov eax, SF_TERMINATE_PROCESS
    int 0x40

_start_window_draw:
    push ebx
    mov eax, SF_REDRAW
    mov ebx, SSF_BEGIN_DRAW
    int 0x40
    pop ebx
    ret

_end_window_draw:
    push ebx
    mov eax, SF_REDRAW
    mov ebx, SSF_END_DRAW
    int 0x40
    pop ebx
    ret

_define_window:
    push ebx
    push edi
    push esi
    mov eax, SF_CREATE_WINDOW
    mov ebx, dword [esp + 4 * 4]
    mov ecx, dword [esp + 4 * 5]
    mov edx, dword [esp + 4 * 6]
    mov esi, dword [esp + 4 * 7]
    mov edi, dword [esp + 4 * 8]
    int 0x40
    pop esi
    pop edi
    pop ebx
    ret

_put_pixel:
    push ebx
    mov eax, SF_PUT_PIXEL
    mov ebx, dword [esp + 4 * 2]
    mov ecx, dword [esp + 4 * 3]
    mov edx, dword [esp + 4 * 4]
    int 0x40
    pop ebx
    ret

_display_message:
    push ebx
    push esi edi
    mov eax, SF_DRAW_TEXT
    mov ebx, dword [esp + 4 * 4]
    mov ecx, dword [esp + 4 * 5]
    mov edx, dword [esp + 4 * 6]
    mov esi, dword [esp + 4 * 7]
    mov edi, dword [esp + 4 * 8]
    int 0x40
    pop edi esi
    pop ebx
    ret

_wait_event:
    mov eax, SF_WAIT_EVENT
    int 0x40
    ret

_pressed_key:
    mov eax, SF_GET_KEY
    int 0x40
    ret

_define_button:
    push ebx
    push esi edi
    mov eax, SF_DEFINE_BUTTON
    mov ebx, dword [esp + 4 * 4]
    mov ecx, dword [esp + 4 * 5]
    mov edx, dword [esp + 4 * 6]
    mov esi, dword [esp + 4 * 7]
    int 0x40
    pop edi esi
    pop ebx
    ret

_debug_write:
    push ebx
    mov eax, SF_BOARD
    mov ebx, SSF_DEBUG_WRITE
    mov cl , byte  [esp + 4 * 2]
    int 0x40
    pop ebx
    ret

_get_button_id:
    mov eax, SF_GET_BUTTON
    int 0x40
    ret

_init_heap:
    push ebx
    mov eax, SF_SYS_MISC
    mov ebx, SSF_HEAP_INIT
    int 0x40
    pop ebx
    ret

_alloc:
    push ebx
    mov eax, SF_SYS_MISC
    mov ebx, SSF_MEM_ALLOC
    mov ecx, [esp + 4 * 2]
    int 0x40
    pop ebx
    ret

_free:
    push ebx
    mov eax, SF_SYS_MISC
    mov ebx, SSF_MEM_FREE
    mov ecx, [esp + 4 * 2]
    int 0x40
    pop ebx
    ret

_get_lang:
    push ebx
    mov eax, SF_SYSTEM_GET
    mov ebx, SSF_SYS_LANG
    int 0x40
    pop ebx
    ret

_load_dll:
    push ebx
    mov eax, SF_SYS_MISC
    mov ebx, SSF_LOAD_DLL
    mov ecx, [esp + 4 * 2]
    int 0x40
    pop ebx
    ret

_set_event_mask:
    push ebx
    mov eax, SF_SET_EVENTS_MASK
    mov ebx, [esp + 4 * 2]
    int 0x40
    pop ebx
    ret

