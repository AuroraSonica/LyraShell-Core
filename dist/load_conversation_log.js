// Updated conversation loading code with support for uploaded images:

async function convertPathToAssetUrl(filePath) {
    if (!filePath) return '';
    
    try {
        const base64Data = await window.__TAURI__.core.invoke('read_file_as_base64', { 
            filePath: filePath 
        });
        
        const fileExtension = filePath.split('.').pop().toLowerCase();
        let mimeType = 'image/jpeg';
        
        if (fileExtension === 'png') mimeType = 'image/png';
        else if (fileExtension === 'webp') mimeType = 'image/webp';
        else if (fileExtension === 'gif') mimeType = 'image/gif';
        
        return `data:${mimeType};base64,${base64Data}`;
        
    } catch (error) {
        console.warn('Failed to load image:', error);
        return '';
    }
}

window.addEventListener("DOMContentLoaded", async () => {
    //console.log("🔍 CONVERSATION LOADING: Starting to load conversation history");
    const chatContainer = document.getElementById("chat-messages");
    //console.log("🔍 CONVERSATION LOADING: Chat container found:", chatContainer);
    
    try {
       // //console.log("🔍 CONVERSATION LOADING: Calling get_conversation_history");
        const conversations = await window.__TAURI__.core.invoke("get_conversation_history");
        //console.log("🔍 CONVERSATION LOADING: Got", conversations.length, "entries");
        
      conversations.forEach((entry, index) => {
            if (entry.includes('💭 Emotional Texture:') || entry.includes('TEXTURE_PLACEHOLDER')) {
                return;
            }

            // Handle the old, separate "🧠 Lyra's Thoughts:" format
            if (entry.includes("🧠 Lyra's Thoughts:")) {
                const lastMessage = chatContainer.lastElementChild;
                if (lastMessage && lastMessage.classList.contains('lyra')) {
                    ensureThinkingStyles();
                    const thinkingProcess = entry.split("🧠 Lyra's Thoughts:")[1]?.trim() || '';
                    const messageId = lastMessage.id;
                    const thinkingId = `thinking_${messageId}`;

                    const thinkingHtml = `
<div class="thinking-header" onclick="(function(){const t=document.getElementById('${thinkingId}');const a=document.getElementById('arrow_${thinkingId}');t.classList.toggle('visible');a.style.transform=t.classList.contains('visible')?'rotate(180deg)':'rotate(0deg)';})()" 
                                 style="height: 24px; padding: 0 10px; background: rgba(0,0,0,0.3); cursor: pointer; display: flex; align-items: center; gap: 6px; color: #9d4edd; box-sizing: border-box;">
                                <span style="font-size: 0.9em; line-height: 1; display: flex; align-items: center;">🧠</span>
                                <strong style="font-size: 0.8em; font-weight: 500; line-height: 1;">Lyra's Thoughts</strong>
                                <span id="arrow_${thinkingId}" style="margin-left: auto; font-size: 0.9em; opacity: 0.7; line-height: 1; transition: transform 0.2s ease;">▼</span>
                            </div>
                            <div id="${thinkingId}" class="thinking-content" style="padding: 8px 12px; font-size: 0.9em; color: #ccc; white-space: pre-wrap; max-height: 250px; overflow-y: auto; border: 2px solid #444; border-bottom-left-radius: 6px; border-bottom-right-radius: 6px; margin-top: -2px;">
                                ${thinkingProcess.replace(/\n/g, '<br>')}
                            </div>
                        </div>`;
                    
                    const messageContentEl = lastMessage.querySelector('.message-content');
                    if (messageContentEl) {
                        messageContentEl.insertAdjacentHTML('beforebegin', thinkingHtml);
                    }
                }
                return;
            }

            if (!entry.includes('[2025-') || !entry.includes('BST]')) {
                return;
            }

            // Handle regular messages (which might contain the new <thinking> format)
            const match = entry.match(/^\[(.*?)\]\s*(🧍|✨|📸|🎵|🎤|👤)\s*([^:]+):\s*(.*)$/s);
            if (match) {
                const [fullMatch, timestamp, emoji, speakerFull, originalMessage] = match;
                
                if (!speakerFull) return;

                let messageContent = originalMessage;
                let thinkingProcess = null;
                if (messageContent && messageContent.includes('<thinking>')) {
                    const thinkingMatch = messageContent.match(/<thinking>([\s\S]*?)<\/thinking>/);
                    if (thinkingMatch) {
                        thinkingProcess = thinkingMatch[1].trim();
                        messageContent = messageContent.replace(/<thinking>[\s\S]*?<\/thinking>/, '').trim();
                    }
                }

                let mainSpeaker = speakerFull.split('(')[0].trim().split('→')[0].trim();
                const senderType = mainSpeaker.toLowerCase() === 'lyra' ? 'lyra' : 'aurora';
                
                const messageDiv = document.createElement('div');
                messageDiv.className = `message ${senderType}`;
                
                const metaText = `${speakerFull || 'System'} • ${timestamp}`;
                const messageId = `hist_msg_${index}`;
                messageDiv.id = messageId;

                let thinkingHtml = '';
                if (thinkingProcess) {
                    ensureThinkingStyles();
                    const thinkingId = `thinking_${messageId}`;
                    thinkingHtml = `
    <div
            class="thinking-header"
            onclick="(function(){const t=document.getElementById('${thinkingId}');const a=document.getElementById('arrow_${thinkingId}');t.classList.toggle('visible');a.style.transform=t.classList.contains('visible')?'rotate(180deg)':'rotate(0deg)';})()"
            style="
                height: 24px;
                padding: 0 10px;
                background: rgba(0,0,0,0.3);
                cursor: pointer;
                display: flex;
                align-items: center;
                gap: 6px;
                color: #9d4edd;
                box-sizing: border-box;
            "
        >
            <span style="font-size: 0.9em; line-height: 1; display: flex; align-items: center;">🧠</span>
            <strong style="font-size: 0.8em; font-weight: 500; line-height: 1;">Lyra's Thoughts</strong>
            <span id="arrow_${thinkingId}" style="margin-left: auto; font-size: 0.9em; opacity: 0.7; line-height: 1; transition: transform 0.2s ease;">▼</span>
        </div>
        <div
            id="${thinkingId}"
            class="thinking-content"
            style="
                padding: 8px 12px;
                font-size: 0.9em;
                color: #ccc;
                white-space: pre-wrap;
                max-height: 250px;
                overflow-y: auto;
                border-top: 2px solid #444;
                border-left: 2px solid #444;
                border-right: 2px solid #444;
                border-bottom: 2px solid #444;
                border-bottom-left-radius: 6px;
                border-bottom-right-radius: 6px;
                margin-top: -2px;
            "
        >
            ${thinkingProcess.replace(/\n/g, '<br>')}
        </div>
    </div>
`;
                }
                
                enhanceMessageContent(messageContent).then(enhancedMessage => {
                    let finalHtml = `
                        <div class="message-meta">${metaText}</div>
                        ${thinkingHtml}
                        <div class="message-content">${enhancedMessage}</div>
                    `;
                    if (senderType === 'lyra') {
                        finalHtml += `<button class="save-memory-btn" onclick="saveToMemory(this, '${messageId}')" style="margin-left: 8px;">💾 Save to Memory</button>`;
                    }
                    messageDiv.innerHTML = finalHtml;
                });
                
                chatContainer.appendChild(messageDiv);
            }
        });
        
        // Scroll to bottom after loading all messages
        setTimeout(() => scrollChatToBottom(false), 100);
        
    } catch (err) {
        console.error("❌ Failed to load conversation history:", err);
    }
});

// Enhanced function to handle both generated and uploaded images
// Function to enhance message content with inline images  
async function enhanceMessageContent(messageContent) {
    // Look for all image patterns: generated, uploaded, and legacy shared
    const imagePatterns = [
        /\[IMAGE: ([^\]]+)\]/g,                    // Generated images
        /\[UPLOADED_IMAGE: ([^\]]+)\]/g,          // New uploaded images
        /\[Shared Image \d+: ([^\]]+)\]/g         // Legacy uploaded format
    ];
    
    let enhancedContent = messageContent;
    
    // Process all patterns
    for (const pattern of imagePatterns) {
        const matches = [...messageContent.matchAll(pattern)];
        
        for (const match of matches) {
            const imagePath = match[1].trim();
            //console.log('🔍 Processing image path:', imagePath);
            
            try {
                let imageType = 'generated';
                let borderColor = '#9d4edd';
                let iconColor = '#9d4edd';
                let typeLabel = 'Lyra\'s creation';
                
                // Detect uploaded images
                if (imagePath.includes('uploaded_images') || imagePath.includes('upload_')) {
                    imageType = 'uploaded';
                    borderColor = '#667eea';
                    iconColor = '#667eea';
                    typeLabel = 'Shared with Lyra';
                }
                
                const imageUrl = await convertPathToAssetUrl(imagePath);
                
                if (imageUrl) {
                    const imageHtml = `
    <div style="margin: 10px 0; padding: 8px; background: rgba(${imageType === 'uploaded' ? '102, 126, 234' : '157, 78, 221'}, 0.1); border-radius: 8px; border-left: 3px solid ${borderColor};">
        <img src="${imageUrl}" 
             style="max-width: 300px; height: auto; border-radius: 8px; cursor: pointer; display: block;"
             onclick="openImageModal('${imagePath.replace(/\\/g, '\\\\')}')"
             alt="${typeLabel}" />
        <div style="font-size: 0.8em; color: ${iconColor}; margin-top: 5px; text-align: center;">
            ${imageType === 'uploaded' ? '📸' : '🎨'} ${typeLabel} • From conversation history
        </div>
    </div>
`;
                    
                    enhancedContent = enhancedContent.replace(match[0], imageHtml);
                } else {
                    // Fallback placeholder
                    const placeholderHtml = `
                        <div style="margin: 10px 0; padding: 8px; background: rgba(255, 164, 77, 0.1); border-radius: 8px; border-left: 3px solid #ffa94d;">
                            <div style="font-size: 0.9em; color: #ffa94d;">
                                ${imageType === 'uploaded' ? '📸' : '🖼️'} ${typeLabel}: ${imagePath.split('\\').pop()}
                            </div>
                            <div style="font-size: 0.8em; color: #ffa94d; opacity: 0.8;">Click Gallery to view</div>
                        </div>
                    `;
                    enhancedContent = enhancedContent.replace(match[0], placeholderHtml);
                }
            } catch (error) {
                console.warn('Failed to enhance image:', imagePath, error);
            }
        }
    }
    
    return enhancedContent;
}

// Helper function to detect image type from path
function detectImageType(imagePath) {
    if (imagePath.includes('uploaded_images') || imagePath.includes('upload_')) {
        return 'uploaded';
    } else if (imagePath.includes('generated_images') || imagePath.includes('LyraShell_')) {
        return 'generated';
    } else {
        return 'unknown';
    }
}